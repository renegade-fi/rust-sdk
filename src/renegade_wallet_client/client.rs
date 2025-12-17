//! The client for interacting with the Renegade darkpool API

use std::sync::Arc;

use alloy::primitives::{keccak256, Address};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::SignerSync;
use ark_ff::PrimeField;
use renegade_circuit_types::schnorr::SchnorrPrivateKey;
use renegade_common::types::tasks::TaskIdentifier;
use renegade_constants::{EmbeddedScalarField, Scalar};
use uuid::Uuid;

use crate::util::get_env_agnostic_chain;
use crate::websocket::{TaskWaiter, TaskWaiterBuilder};
use crate::{
    http::RelayerHttpClient, renegade_wallet_client::config::RenegadeClientConfig, util::HmacKey,
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
#[derive(Clone)]
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

        let chain = get_env_agnostic_chain(config.chain_id);
        let historical_state_client = Arc::new(RelayerHttpClient::new(
            format!("{}/{chain}", config.historical_state_base_url),
            secrets.auth_hmac_key,
        ));

        let websocket_client =
            RenegadeWebsocketClient::new(&config, secrets.account_id, secrets.auth_hmac_key);

        Ok(Self { config, secrets, relayer_client, historical_state_client, websocket_client })
    }

    /// Create a new wallet on Arbitrum Sepolia
    pub fn new_arbitrum_sepolia(key: &PrivateKeySigner) -> Result<Self, RenegadeClientError> {
        Self::new(RenegadeClientConfig::new_arbitrum_sepolia(key))
    }

    /// Create a new wallet on Arbitrum One
    pub fn new_arbitrum_one(key: &PrivateKeySigner) -> Result<Self, RenegadeClientError> {
        Self::new(RenegadeClientConfig::new_arbitrum_one(key))
    }

    /// Create a new wallet on Base Sepolia
    pub fn new_base_sepolia(key: &PrivateKeySigner) -> Result<Self, RenegadeClientError> {
        Self::new(RenegadeClientConfig::new_base_sepolia(key))
    }

    /// Create a new wallet on Base Mainnet
    pub fn new_base_mainnet(key: &PrivateKeySigner) -> Result<Self, RenegadeClientError> {
        Self::new(RenegadeClientConfig::new_base_mainnet(key))
    }

    /// Whether the client is on a chain in which Renegade is deployed as a
    /// solidity contract
    pub fn is_solidity_chain(&self) -> bool {
        self.config.chain_id == BASE_MAINNET_CHAIN_ID
            || self.config.chain_id == BASE_SEPOLIA_CHAIN_ID
    }

    // --------------
    // | Task Utils |
    // --------------

    /// Get a task waiter builder for a task
    pub fn get_task_waiter_builder(&self, task_id: TaskIdentifier) -> TaskWaiterBuilder {
        TaskWaiterBuilder::new(task_id, self.websocket_client.clone())
    }

    /// Get a default-configured task waiter for a task
    pub fn get_default_task_waiter(&self, task_id: TaskIdentifier) -> TaskWaiter {
        self.get_task_waiter_builder(task_id).build()
    }

    // --------------
    // | Misc Utils |
    // --------------

    /// Get the address of the account associated with the private key the
    /// client is configured with
    pub fn get_account_address(&self) -> Address {
        self.config.key.address()
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
