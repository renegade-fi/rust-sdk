//! The client for interacting with the Renegade darkpool API

use alloy::signers::local::PrivateKeySigner;
use renegade_common::types::wallet::{
    derivation::{
        derive_blinder_seed, derive_share_seed, derive_wallet_id, derive_wallet_keychain,
    },
    keychain::KeyChain,
    Wallet,
};
use renegade_constants::Scalar;
use reqwest::header::HeaderMap;
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

use crate::{
    http::RelayerHttpClient,
    renegade_wallet_client::config::{
        RenegadeClientConfig, BASE_MAINNET_CHAIN_ID, BASE_SEPOLIA_CHAIN_ID,
    },
    util::HmacKey as HttpHmacKey,
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
}

impl RenegadeClient {
    /// Derive the wallet secrets from an ethereum private key
    pub fn new(config: RenegadeClientConfig) -> Result<Self, RenegadeClientError> {
        let secrets = derive_wallet_from_key(&config.key, config.chain_id)
            .map_err(RenegadeClientError::setup)?;
        let hmac_key = secrets.keychain.secret_keys.symmetric_key;
        let client =
            RelayerHttpClient::new(config.relayer_base_url.clone(), HttpHmacKey(hmac_key.0));

        Ok(Self { config, secrets, relayer_client: client })
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
    // | HTTP Utils |
    // --------------

    /// Send a get request to the relayer
    pub async fn get_relayer<Resp: DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<Resp, RenegadeClientError> {
        let headers = HeaderMap::new();
        let resp = self
            .relayer_client
            .get_with_headers_raw(path, headers)
            .await
            .map_err(RenegadeClientError::request)?;
        let body = resp.text().await.unwrap_or_else(|_| RESPONSE_BODY_DECODE_ERROR.to_string());

        // Try decoding the response body as the expected type
        let decoded: Result<Resp, _> = serde_json::from_str(&body);
        if let Ok(decoded) = decoded {
            Ok(decoded)
        } else {
            Err(RenegadeClientError::relayer(body))
        }
    }

    /// Send a post request to the relayer
    pub async fn post_relayer<Req: Serialize, Resp: DeserializeOwned>(
        &self,
        path: &str,
        body: Req,
    ) -> Result<Resp, RenegadeClientError> {
        // Send an HTTP request to the relayer
        let headers = HeaderMap::new();
        let resp = self
            .relayer_client
            .post_with_headers_raw(path, body, headers)
            .await
            .map_err(RenegadeClientError::request)?;
        let body = resp.text().await.unwrap_or_else(|_| RESPONSE_BODY_DECODE_ERROR.to_string());

        // Attempt to decode the response body as the expected type
        // Otherwise, emit the body as an error
        let decoded: Result<Resp, _> = serde_json::from_str(&body);
        if let Ok(decoded) = decoded {
            Ok(decoded)
        } else {
            Err(RenegadeClientError::relayer(body))
        }
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
