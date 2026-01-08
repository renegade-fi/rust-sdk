//! The client for interacting with the Renegade darkpool API

use std::sync::Arc;
use std::time::Duration;

use alloy::primitives::{keccak256, Address};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::SignerSync;
use ark_ff::PrimeField;
use futures_util::Stream;
use renegade_circuit_types::schnorr::{SchnorrPrivateKey, SchnorrPublicKey, SchnorrSignature};
use renegade_circuit_types::traits::BaseType;
use renegade_constants::{EmbeddedScalarField, Scalar};
use renegade_types_core::HmacKey;
use uuid::Uuid;

use crate::renegade_api_types::tasks::TaskIdentifier;
use crate::renegade_api_types::websocket::{
    AdminBalanceUpdateWebsocketMessage, AdminOrderUpdateWebsocketMessage,
    BalanceUpdateWebsocketMessage, FillWebsocketMessage, OrderUpdateWebsocketMessage,
    TaskUpdateWebsocketMessage,
};
use crate::util::get_env_agnostic_chain;
use crate::websocket::TaskWaiter;
use crate::{
    http::RelayerHttpClient, renegade_wallet_client::config::RenegadeClientConfig,
    websocket::RenegadeWebsocketClient, RenegadeClientError, BASE_MAINNET_CHAIN_ID,
    BASE_SEPOLIA_CHAIN_ID,
};

// -------------
// | Constants |
// -------------

/// The message prefix used to derive the account ID
const ACCOUNT_ID_MESSAGE_PREFIX: &[u8] = b"account id";
/// The message prefix used to derive the master view seed
const MASTER_VIEW_SEED_MESSAGE_PREFIX: &[u8] = b"master view seed";
/// The message prefix used to derive the schnorr key
const SCHNORR_KEY_MESSAGE_PREFIX: &[u8] = b"schnorr key";
/// The message prefix used to derive the auth HMAC key
const AUTH_HMAC_KEY_MESSAGE_PREFIX: &[u8] = b"auth hmac key";

/// The number of bytes from a keccak hash
const KECCAK_HASH_BYTES: usize = 32;
/// The number of bytes we extend into to get a scalar
const EXTENDED_BYTES: usize = 64;
/// The number of bytes in an account ID
const ACCOUNT_ID_BYTES: usize = 16;

// -----------
// | Secrets |
// -----------

/// The secrets used to authenticate account actions
#[derive(Copy, Clone)]
pub struct AccountSecrets {
    /// The ID of the account
    pub account_id: Uuid,
    /// The master view seed, used to sync the account with onchain state &
    /// derive CSPRNG seeds for new state objects
    pub master_view_seed: Scalar,
    /// The private key used for Schnorr signatures over state objects
    pub schnorr_key: SchnorrPrivateKey,
    /// The HMAC key used to authenticate account API actions
    pub auth_hmac_key: HmacKey,
}

impl AccountSecrets {
    /// Generate a new set of account secrets from a signing key & chain ID
    pub fn new(key: &PrivateKeySigner, chain_id: u64) -> Result<Self, RenegadeClientError> {
        let account_id = derive_account_id(key, chain_id).map_err(RenegadeClientError::setup)?;

        let master_view_seed =
            derive_master_view_seed(key, chain_id).map_err(RenegadeClientError::setup)?;

        let schnorr_key = derive_schnorr_key(key, chain_id).map_err(RenegadeClientError::setup)?;

        let auth_hmac_key =
            derive_auth_hmac_key(key, chain_id).map_err(RenegadeClientError::setup)?;

        Ok(Self { account_id, master_view_seed, schnorr_key, auth_hmac_key })
    }
}

// -------------------
// | Darkpool Client |
// -------------------

/// The Renegade wallet client
#[derive(Clone)]
pub struct RenegadeClient {
    /// The client config
    pub config: RenegadeClientConfig,
    /// The account secrets
    pub secrets: AccountSecrets,
    /// The relayer HTTP client
    pub relayer_client: RelayerHttpClient,
    /// The admin relayer HTTP client
    pub admin_relayer_client: Option<RelayerHttpClient>,
    /// The historical state HTTP client.
    ///
    /// Also a `RelayerHttpClient` as it mirrors the relayer's historical state
    /// API.
    pub historical_state_client: Arc<RelayerHttpClient>,
    /// The websocket client
    pub websocket_client: RenegadeWebsocketClient,
}

impl RenegadeClient {
    /// Derive the wallet secrets from an ethereum private key
    pub fn new(config: RenegadeClientConfig) -> Result<Self, RenegadeClientError> {
        let secrets = AccountSecrets::new(&config.key, config.chain_id)?;

        let relayer_client =
            RelayerHttpClient::new(config.relayer_base_url.clone(), secrets.auth_hmac_key);

        let admin_relayer_client = config
            .admin_hmac_key
            .map(|key| RelayerHttpClient::new(config.relayer_base_url.clone(), key));

        let chain = get_env_agnostic_chain(config.chain_id);
        let historical_state_client = Arc::new(RelayerHttpClient::new(
            format!("{}/{chain}", config.historical_state_base_url),
            secrets.auth_hmac_key,
        ));

        let websocket_client = RenegadeWebsocketClient::new(
            &config,
            secrets.account_id,
            secrets.auth_hmac_key,
            config.admin_hmac_key,
        );

        Ok(Self {
            config,
            secrets,
            relayer_client,
            admin_relayer_client,
            historical_state_client,
            websocket_client,
        })
    }

    /// Create a new wallet on Arbitrum Sepolia
    pub fn new_arbitrum_sepolia(key: &PrivateKeySigner) -> Result<Self, RenegadeClientError> {
        Self::new(RenegadeClientConfig::new_arbitrum_sepolia(key))
    }

    /// Create a new admin wallet on Arbitrum Sepolia
    pub fn new_arbitrum_sepolia_admin(
        key: &PrivateKeySigner,
        admin_hmac_key: HmacKey,
    ) -> Result<Self, RenegadeClientError> {
        Self::new(RenegadeClientConfig::new_arbitrum_sepolia_admin(key, admin_hmac_key))
    }

    /// Create a new wallet on Arbitrum One
    pub fn new_arbitrum_one(key: &PrivateKeySigner) -> Result<Self, RenegadeClientError> {
        Self::new(RenegadeClientConfig::new_arbitrum_one(key))
    }

    /// Create a new admin wallet on Arbitrum One
    pub fn new_arbitrum_one_admin(
        key: &PrivateKeySigner,
        admin_hmac_key: HmacKey,
    ) -> Result<Self, RenegadeClientError> {
        Self::new(RenegadeClientConfig::new_arbitrum_one_admin(key, admin_hmac_key))
    }

    /// Create a new wallet on Base Sepolia
    pub fn new_base_sepolia(key: &PrivateKeySigner) -> Result<Self, RenegadeClientError> {
        Self::new(RenegadeClientConfig::new_base_sepolia(key))
    }

    /// Create a new admin wallet on Base Sepolia
    pub fn new_base_sepolia_admin(
        key: &PrivateKeySigner,
        admin_hmac_key: HmacKey,
    ) -> Result<Self, RenegadeClientError> {
        Self::new(RenegadeClientConfig::new_base_sepolia_admin(key, admin_hmac_key))
    }

    /// Create a new wallet on Base Mainnet
    pub fn new_base_mainnet(key: &PrivateKeySigner) -> Result<Self, RenegadeClientError> {
        Self::new(RenegadeClientConfig::new_base_mainnet(key))
    }

    /// Create a new admin wallet on Base Mainnet
    pub fn new_base_mainnet_admin(
        key: &PrivateKeySigner,
        admin_hmac_key: HmacKey,
    ) -> Result<Self, RenegadeClientError> {
        Self::new(RenegadeClientConfig::new_base_mainnet_admin(key, admin_hmac_key))
    }

    /// Whether the client is on a chain in which Renegade is deployed as a
    /// solidity contract
    pub fn is_solidity_chain(&self) -> bool {
        self.config.chain_id == BASE_MAINNET_CHAIN_ID
            || self.config.chain_id == BASE_SEPOLIA_CHAIN_ID
    }

    // --------------
    // | WS Methods |
    // --------------

    /// Create a `TaskWaiter` which can be used to watch a task until it
    /// completes or times out
    pub async fn watch_task(
        &self,
        task_id: TaskIdentifier,
        timeout: Duration,
    ) -> Result<TaskWaiter, RenegadeClientError> {
        self.websocket_client.watch_task(task_id, timeout).await
    }

    /// Subscribe to the account's task updates stream
    pub async fn subscribe_task_updates(
        &self,
    ) -> Result<impl Stream<Item = TaskUpdateWebsocketMessage>, RenegadeClientError> {
        self.websocket_client.subscribe_task_updates().await
    }

    /// Subscribe to the account's balance updates stream
    pub async fn subscribe_balance_updates(
        &self,
    ) -> Result<impl Stream<Item = BalanceUpdateWebsocketMessage>, RenegadeClientError> {
        self.websocket_client.subscribe_balance_updates().await
    }

    /// Subscribe to the account's order updates stream
    pub async fn subscribe_order_updates(
        &self,
    ) -> Result<impl Stream<Item = OrderUpdateWebsocketMessage>, RenegadeClientError> {
        self.websocket_client.subscribe_order_updates().await
    }

    /// Subscribe to the account's fills stream
    pub async fn subscribe_fills(
        &self,
    ) -> Result<impl Stream<Item = FillWebsocketMessage>, RenegadeClientError> {
        self.websocket_client.subscribe_fills().await
    }

    /// Subscribe to the admin balances updates stream
    pub async fn subscribe_admin_balance_updates(
        &self,
    ) -> Result<impl Stream<Item = AdminBalanceUpdateWebsocketMessage>, RenegadeClientError> {
        self.websocket_client.subscribe_admin_balance_updates().await
    }

    /// Subscribe to the admin order updates stream
    pub async fn subscribe_admin_order_updates(
        &self,
    ) -> Result<impl Stream<Item = AdminOrderUpdateWebsocketMessage>, RenegadeClientError> {
        self.websocket_client.subscribe_admin_order_updates().await
    }

    // --------------
    // | Misc Utils |
    // --------------

    /// Get a reference to the admin relayer client, returning an error if one
    /// has not been configured.
    pub fn get_admin_client(&self) -> Result<&RelayerHttpClient, RenegadeClientError> {
        match self.admin_relayer_client.as_ref() {
            Some(admin_client) => Ok(admin_client),
            None => Err(RenegadeClientError::NotAdmin),
        }
    }

    /// Get the ID of the account
    pub fn get_account_id(&self) -> Uuid {
        self.secrets.account_id
    }

    /// Get the master view seed
    pub fn get_master_view_seed(&self) -> Scalar {
        self.secrets.master_view_seed
    }

    /// Get the HMAC key used to authenticate account API actions
    pub fn get_auth_hmac_key(&self) -> HmacKey {
        self.secrets.auth_hmac_key
    }

    /// Get the signing key client is configured with
    pub fn get_account_signer(&self) -> &PrivateKeySigner {
        &self.config.key
    }

    /// Get the address of the account associated with the private key the
    /// client is configured with
    pub fn get_account_address(&self) -> Address {
        self.config.key.address()
    }

    /// Get the Schnorr private key client is configured with
    pub fn schnorr_sign<T: BaseType>(
        &self,
        message: &T,
    ) -> Result<SchnorrSignature, RenegadeClientError> {
        self.secrets.schnorr_key.sign(message).map_err(RenegadeClientError::signing)
    }

    /// Get the public key associated with the Schnorr private key the client is
    /// configured with
    pub fn get_schnorr_public_key(&self) -> SchnorrPublicKey {
        self.secrets.schnorr_key.public_key()
    }

    /// Get the relayer's executor address, which it uses to sign public order
    /// settlement obligations
    pub fn get_executor_address(&self) -> Address {
        self.config.executor_address
    }

    /// Get the relayer's fee recipient address
    pub fn get_relayer_fee_recipient(&self) -> Address {
        self.config.relayer_fee_recipient
    }

    /// Get the chain ID the client is configured for
    pub fn get_chain_id(&self) -> u64 {
        self.config.chain_id
    }

    /// Get the permit2 address the client is configured for
    pub fn get_permit2_address(&self) -> Address {
        self.config.permit2_address
    }

    /// Get the darkpool address the client is configured for
    pub fn get_darkpool_address(&self) -> Address {
        self.config.darkpool_address
    }
}

// -----------
// | Helpers |
// -----------

/// Derive an account ID from a signing key & chain ID
fn derive_account_id(key: &PrivateKeySigner, chain_id: u64) -> Result<Uuid, String> {
    let sig_bytes = get_sig_bytes(&account_id_message(chain_id), key)?;
    let account_id = Uuid::from_slice(&sig_bytes[..ACCOUNT_ID_BYTES])
        .map_err(|e| format!("failed to derive account ID: {e}"))?;
    Ok(account_id)
}

/// Derive a master view seed from a signing key & chain ID
fn derive_master_view_seed(key: &PrivateKeySigner, chain_id: u64) -> Result<Scalar, String> {
    let sig_bytes = get_extended_sig_bytes(&master_view_seed_message(chain_id), key)?;
    Ok(Scalar::from_be_bytes_mod_order(&sig_bytes))
}

/// Derive a Schnorr private key from a signing key & chain ID
fn derive_schnorr_key(key: &PrivateKeySigner, chain_id: u64) -> Result<SchnorrPrivateKey, String> {
    let sig_bytes = get_extended_sig_bytes(&schnorr_key_message(chain_id), key)?;
    let inner = EmbeddedScalarField::from_be_bytes_mod_order(&sig_bytes);
    Ok(SchnorrPrivateKey { inner })
}

/// Derive an HMAC key for API auth from a signing key & chain ID
fn derive_auth_hmac_key(key: &PrivateKeySigner, chain_id: u64) -> Result<HmacKey, String> {
    get_sig_bytes(&auth_hmac_key_message(chain_id), key).map(HmacKey)
}

/// Generate the message to sign for account ID derivation
fn account_id_message(chain_id: u64) -> Vec<u8> {
    let mut message = Vec::from(ACCOUNT_ID_MESSAGE_PREFIX);
    message.extend_from_slice(&chain_id.to_be_bytes());
    message
}

/// Generate the message to sign for master view seed derivation
fn master_view_seed_message(chain_id: u64) -> Vec<u8> {
    let mut message = Vec::from(MASTER_VIEW_SEED_MESSAGE_PREFIX);
    message.extend_from_slice(&chain_id.to_be_bytes());
    message
}

/// Generate the message to sign for Schnorr key derivation
fn schnorr_key_message(chain_id: u64) -> Vec<u8> {
    let mut message = Vec::from(SCHNORR_KEY_MESSAGE_PREFIX);
    message.extend_from_slice(&chain_id.to_be_bytes());
    message
}

/// Generate the message to sign for auth HMAC key derivation
fn auth_hmac_key_message(chain_id: u64) -> Vec<u8> {
    let mut message = Vec::from(AUTH_HMAC_KEY_MESSAGE_PREFIX);
    message.extend_from_slice(&chain_id.to_be_bytes());
    message
}

/// Sign a message, serialize the signature into bytes
fn get_sig_bytes(msg: &[u8], key: &PrivateKeySigner) -> Result<[u8; KECCAK_HASH_BYTES], String> {
    let digest = keccak256(msg);
    let sig = key.sign_hash_sync(&digest).map_err(|e| format!("failed to sign message: {e}"))?;

    // Take the keccak hash of the signature to disperse its elements
    let bytes: Vec<u8> = sig.into();
    Ok(*keccak256(bytes))
}

/// Sign a message, serialize the signature into bytes, and extend the bytes to
/// support secure reduction into a field
fn get_extended_sig_bytes(
    msg: &[u8],
    key: &PrivateKeySigner,
) -> Result<[u8; EXTENDED_BYTES], String> {
    let sig_bytes = get_sig_bytes(msg, key)?;
    Ok(extend_to_64_bytes(&sig_bytes))
}

/// Extend the given byte array to 64 bytes, double the length of the original
///
/// This is necessary to give a uniform sampling of a field that these bytes are
/// reduced into, the bitlength must be significantly larger than the field's
/// bitlength to avoid sample bias via modular reduction
fn extend_to_64_bytes(bytes: &[u8]) -> [u8; EXTENDED_BYTES] {
    let mut extended = [0; EXTENDED_BYTES];
    let top_bytes = keccak256(bytes);
    extended[..KECCAK_HASH_BYTES].copy_from_slice(bytes);
    extended[KECCAK_HASH_BYTES..].copy_from_slice(&top_bytes.0);
    extended
}
