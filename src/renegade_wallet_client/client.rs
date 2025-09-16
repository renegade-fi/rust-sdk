//! The client for interacting with the Renegade darkpool API

use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use renegade_common::types::tasks::TaskIdentifier;
use renegade_common::types::wallet::{
    derivation::{
        derive_blinder_seed, derive_share_seed, derive_wallet_id, derive_wallet_keychain,
    },
    keychain::KeyChain,
    Wallet,
};
use renegade_constants::Scalar;
use uuid::Uuid;

use crate::websocket::TaskWaiter;
use crate::{
    http::RelayerHttpClient,
    renegade_wallet_client::config::{
        RenegadeClientConfig, BASE_MAINNET_CHAIN_ID, BASE_SEPOLIA_CHAIN_ID,
    },
    util::HmacKey as HttpHmacKey,
    websocket::RenegadeWebsocketClient,
    RenegadeClientError,
};

// -------------
// | Constants |
// -------------

/// The error message when a response body cannot be decoded
const RESPONSE_BODY_DECODE_ERROR: &str = "<failed to decode response body>";

// -----------
// | Secrets |
// -----------

/// The secrets used to authenticate and fetch a wallet
#[derive(Debug, Clone)]
pub struct WalletSecrets {
    /// The ID of the wallet
    pub wallet_id: Uuid,
    /// The wallet's blinder seed
    pub blinder_seed: Scalar,
    /// The wallet's share seed
    pub share_seed: Scalar,
    /// The wallet's keychain
    pub keychain: KeyChain,
}

impl WalletSecrets {
    /// Generate an empty wallet with the given set of secrets
    pub fn generate_empty_wallet(&self) -> Wallet {
        Wallet::new_empty_wallet(
            self.wallet_id,
            self.blinder_seed,
            self.share_seed,
            self.keychain.clone(),
        )
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
    /// The wallet secrets
    pub secrets: WalletSecrets,
    /// The relayer HTTP client
    pub relayer_client: RelayerHttpClient,
    /// The historical state HTTP client.
    ///
    /// Also a `RelayerHttpClient` as it mirrors the relayer's historical state
    /// API.
    pub historical_state_client: RelayerHttpClient,
    /// The websocket client
    pub websocket_client: RenegadeWebsocketClient,
}

impl RenegadeClient {
    /// Derive the wallet secrets from an ethereum private key
    pub fn new(config: RenegadeClientConfig) -> Result<Self, RenegadeClientError> {
        let secrets = derive_wallet_from_key(&config.key, config.chain_id)
            .map_err(RenegadeClientError::setup)?;
        let hmac_key = secrets.keychain.secret_keys.symmetric_key;

        let relayer_client =
            RelayerHttpClient::new(config.relayer_base_url.clone(), HttpHmacKey(hmac_key.0));

        let historical_state_client = RelayerHttpClient::new(
            config.historical_state_base_url.clone(),
            HttpHmacKey(hmac_key.0),
        );

        let websocket_client = RenegadeWebsocketClient::new(&config);

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

    /// Get a task waiter for a task
    pub fn get_task_waiter(&self, task_id: TaskIdentifier) -> TaskWaiter {
        TaskWaiter::new(task_id, self.websocket_client.clone())
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

/// Derive a new wallet from a private key
///
/// Returns the wallet, the blinder seed, and the share seed
fn derive_wallet_from_key(
    root_key: &PrivateKeySigner,
    chain_id: u64,
) -> Result<WalletSecrets, String> {
    // Derive the seeds and keychain
    let wallet_id = derive_wallet_id(root_key)?;
    let blinder_seed = derive_blinder_seed(root_key)?;
    let share_seed = derive_share_seed(root_key)?;
    let keychain = derive_wallet_keychain(root_key, chain_id)?;

    Ok(WalletSecrets { wallet_id, blinder_seed, share_seed, keychain })
}
