//! A client for interacting with the darkpool

use ethers::signers::LocalWallet;
use eyre::Result;
use renegade_common::types::wallet::{
    derivation::{
        derive_blinder_seed, derive_share_seed, derive_wallet_id, derive_wallet_keychain,
    },
    keychain::KeyChain,
    WalletIdentifier,
};
use renegade_constants::Scalar;

use crate::{http::RelayerHttpClient, wrap_eyre};

mod wallet_ops;

/// The Arbitrum Sepolia chain ID
const ARBITRUM_SEPOLIA_CHAIN_ID: u64 = 421611;
/// The Arbitrum mainnet chain ID
const ARBITRUM_MAINNET_CHAIN_ID: u64 = 42161;
/// The base URL for the Renegade relayer in testnet
const TESTNET_BASE_URL: &str = "https://testnet.cluster0.renegade.fi:3000";
/// The base URL for the Renegade relayer in mainnet
const MAINNET_BASE_URL: &str = "https://mainnet.cluster0.renegade.fi:3000";

/// The secrets held by a wallet
#[derive(Clone)]
pub struct WalletSecrets {
    /// The wallet's ID
    wallet_id: WalletIdentifier,
    /// The wallet's blinder seed
    blinder_seed: Scalar,
    /// The wallet's secret share seed
    share_seed: Scalar,
    /// The wallet keychain
    keychain: KeyChain,
}

impl WalletSecrets {
    /// Derive a set of wallet secrets from an ethereum private key
    pub fn from_ethereum_pkey(chain_id: u64, pkey: &LocalWallet) -> Result<Self> {
        // Derive the seeds and keychain
        let wallet_id = wrap_eyre!(derive_wallet_id(pkey))?;
        let blinder_seed = wrap_eyre!(derive_blinder_seed(pkey))?;
        let share_seed = wrap_eyre!(derive_share_seed(pkey))?;
        let keychain = wrap_eyre!(derive_wallet_keychain(pkey, chain_id))?;

        Ok(Self { wallet_id, blinder_seed, share_seed, keychain })
    }
}

/// The darkpool client for interacting with a Renegade relayer
#[derive(Clone)]
pub struct DarkpoolClient {
    /// The wallet secrets
    wallet_secrets: WalletSecrets,
    /// The HTTP client
    http_client: RelayerHttpClient,
}

impl DarkpoolClient {
    /// Create a new darkpool client using the given Ethereum private key to
    /// derive the wallet secrets
    pub fn new(chain_id: u64, base_url: &str, pkey: &LocalWallet) -> Result<Self> {
        // Derive the wallet secrets
        let wallet_secrets = wrap_eyre!(WalletSecrets::from_ethereum_pkey(chain_id, pkey))?;

        // Create the client
        let hmac_key = wallet_secrets.keychain.secret_keys.symmetric_key;
        let http_client = RelayerHttpClient::new(base_url.to_string(), hmac_key);
        Ok(Self { wallet_secrets, http_client })
    }

    /// Create a new Arbitrum Sepolia darkpool client
    pub fn new_arbitrum_sepolia(pkey: &LocalWallet) -> Result<Self> {
        Self::new(ARBITRUM_SEPOLIA_CHAIN_ID, TESTNET_BASE_URL, pkey)
    }

    /// Create a new Arbitrum mainnet darkpool client
    pub fn new_arbitrum_mainnet(pkey: &LocalWallet) -> Result<Self> {
        Self::new(ARBITRUM_MAINNET_CHAIN_ID, MAINNET_BASE_URL, pkey)
    }
}
