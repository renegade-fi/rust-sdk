//! The client for interacting with the Renegade darkpool API

use alloy::signers::local::PrivateKeySigner;
use renegade_common::types::wallet::{
    derivation::{
        derive_blinder_seed, derive_share_seed, derive_wallet_id, derive_wallet_keychain,
    },
    keychain::KeyChain,
};
use renegade_constants::Scalar;
use uuid::Uuid;

use crate::RenegadeClientError;

/// The Arbitrum one chain ID
const ARBITRUM_ONE_CHAIN_ID: u64 = 42161;
/// The Arbitrum Sepolia chain ID
const ARBITRUM_SEPOLIA_CHAIN_ID: u64 = 421614;
/// The Base mainnet chain ID
const BASE_MAINNET_CHAIN_ID: u64 = 8453;
/// The Base Sepolia chain ID
const BASE_SEPOLIA_CHAIN_ID: u64 = 84532;

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

// -------------------
// | Darkpool Client |
// -------------------

/// The Renegade wallet client
#[derive(Debug, Clone)]
pub struct RenegadeClient {
    /// The wallet secrets
    pub secrets: WalletSecrets,
}

impl RenegadeClient {
    /// Derive the wallet secrets from an ethereum private key
    pub fn new(key: &PrivateKeySigner, chain_id: u64) -> Result<Self, RenegadeClientError> {
        let secrets =
            derive_wallet_from_key(key, chain_id).map_err(RenegadeClientError::setup_error)?;
        Ok(Self { secrets })
    }

    /// Create a new wallet on Arbitrum Sepolia
    pub fn new_arbitrum_sepolia(key: &PrivateKeySigner) -> Result<Self, RenegadeClientError> {
        Self::new(key, ARBITRUM_SEPOLIA_CHAIN_ID)
    }

    /// Create a new wallet on Arbitrum One
    pub fn new_arbitrum_one(key: &PrivateKeySigner) -> Result<Self, RenegadeClientError> {
        Self::new(key, ARBITRUM_ONE_CHAIN_ID)
    }

    /// Create a new wallet on Base Sepolia
    pub fn new_base_sepolia(key: &PrivateKeySigner) -> Result<Self, RenegadeClientError> {
        Self::new(key, BASE_SEPOLIA_CHAIN_ID)
    }

    /// Create a new wallet on Base Mainnet
    pub fn new_base_mainnet(key: &PrivateKeySigner) -> Result<Self, RenegadeClientError> {
        Self::new(key, BASE_MAINNET_CHAIN_ID)
    }
}

// -----------
// | Helpers |
// -----------

/// Derive a new wallet from a private key
///
/// Returns the wallet, the blinder seed, and the share seed
pub fn derive_wallet_from_key(
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
