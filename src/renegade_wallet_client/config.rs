//! Config setup for the renegade wallet client

// -------------
// | Constants |
// -------------

use alloy::{
    primitives::{address, Address},
    signers::local::PrivateKeySigner,
};

use crate::{
    ARBITRUM_ONE_CHAIN_ID, ARBITRUM_ONE_RELAYER_BASE_URL, ARBITRUM_SEPOLIA_CHAIN_ID,
    ARBITRUM_SEPOLIA_RELAYER_BASE_URL, BASE_MAINNET_CHAIN_ID, BASE_MAINNET_RELAYER_BASE_URL,
    BASE_SEPOLIA_CHAIN_ID, BASE_SEPOLIA_RELAYER_BASE_URL,
};

// --- Historical State URLs --- //

/// The mainnet historical state base URL
pub(crate) const MAINNET_HISTORICAL_STATE_BASE_URL: &str =
    "https://mainnet.historical-state.renegade.fi";
/// The testnet historical state base URL
pub(crate) const TESTNET_HISTORICAL_STATE_BASE_URL: &str =
    "https://testnet.historical-state.renegade.fi";

// --- Darkpool Addresses --- //

/// The darkpool address on Arbitrum One
pub(crate) const ARBITRUM_ONE_DARKPOOL_ADDRESS: Address =
    address!("0x30bd8eab29181f790d7e495786d4b96d7afdc518");
/// The darkpool address on Arbitrum Sepolia
pub(crate) const ARBITRUM_SEPOLIA_DARKPOOL_ADDRESS: Address =
    address!("0x9af58f1ff20ab22e819e40b57ffd784d115a9ef5");
/// The darkpool address on Base Mainnet
pub(crate) const BASE_MAINNET_DARKPOOL_ADDRESS: Address =
    address!("0xb4a96068577141749CC8859f586fE29016C935dB");
/// The darkpool address on Base Sepolia
pub(crate) const BASE_SEPOLIA_DARKPOOL_ADDRESS: Address =
    address!("0x653C95391644EEE16E4975a7ef1f46e0B8276695");

// --- Permit2 Addresses --- //

/// The permit2 address on Arbitrum One
pub(crate) const ARBITRUM_ONE_PERMIT2_ADDRESS: Address =
    address!("0x000000000022D473030F116dDEE9F6B43aC78BA3");
/// The permit2 address on Arbitrum Sepolia
pub(crate) const ARBITRUM_SEPOLIA_PERMIT2_ADDRESS: Address =
    address!("0x9458198bcc289c42e460cb8ca143e5854f734442");
/// The permit2 address on Base Mainnet
pub(crate) const BASE_MAINNET_PERMIT2_ADDRESS: Address =
    address!("0x000000000022D473030F116dDEE9F6B43aC78BA3");
/// The permit2 address on Base Sepolia
pub(crate) const BASE_SEPOLIA_PERMIT2_ADDRESS: Address =
    address!("0x000000000022D473030F116dDEE9F6B43aC78BA3");

/// The client config
#[derive(Debug, Clone)]
pub struct RenegadeClientConfig {
    /// The relayer base URL
    pub relayer_base_url: String,
    /// The historical state base URL
    pub historical_state_base_url: String,
    /// The chain ID
    pub chain_id: u64,
    /// The darkpool contract address
    pub darkpool_address: Address,
    /// The permit2 contract address
    pub permit2_address: Address,
    /// The private key from which to derive the wallet
    pub key: PrivateKeySigner,
}

impl RenegadeClientConfig {
    /// Create a new client config for Arbitrum One
    pub fn new_arbitrum_one(key: &PrivateKeySigner) -> Self {
        Self {
            relayer_base_url: ARBITRUM_ONE_RELAYER_BASE_URL.to_string(),
            historical_state_base_url: MAINNET_HISTORICAL_STATE_BASE_URL.to_string(),
            chain_id: ARBITRUM_ONE_CHAIN_ID,
            darkpool_address: ARBITRUM_ONE_DARKPOOL_ADDRESS,
            permit2_address: ARBITRUM_ONE_PERMIT2_ADDRESS,
            key: key.clone(),
        }
    }

    /// Create a new client config for Arbitrum Sepolia
    pub fn new_arbitrum_sepolia(key: &PrivateKeySigner) -> Self {
        Self {
            relayer_base_url: ARBITRUM_SEPOLIA_RELAYER_BASE_URL.to_string(),
            historical_state_base_url: TESTNET_HISTORICAL_STATE_BASE_URL.to_string(),
            chain_id: ARBITRUM_SEPOLIA_CHAIN_ID,
            darkpool_address: ARBITRUM_SEPOLIA_DARKPOOL_ADDRESS,
            permit2_address: ARBITRUM_SEPOLIA_PERMIT2_ADDRESS,
            key: key.clone(),
        }
    }

    /// Create a new client config for Base Mainnet
    pub fn new_base_mainnet(key: &PrivateKeySigner) -> Self {
        Self {
            relayer_base_url: BASE_MAINNET_RELAYER_BASE_URL.to_string(),
            historical_state_base_url: MAINNET_HISTORICAL_STATE_BASE_URL.to_string(),
            chain_id: BASE_MAINNET_CHAIN_ID,
            darkpool_address: BASE_MAINNET_DARKPOOL_ADDRESS,
            permit2_address: BASE_MAINNET_PERMIT2_ADDRESS,
            key: key.clone(),
        }
    }

    /// Create a new client config for Base Sepolia
    pub fn new_base_sepolia(key: &PrivateKeySigner) -> Self {
        Self {
            relayer_base_url: BASE_SEPOLIA_RELAYER_BASE_URL.to_string(),
            historical_state_base_url: TESTNET_HISTORICAL_STATE_BASE_URL.to_string(),
            chain_id: BASE_SEPOLIA_CHAIN_ID,
            darkpool_address: BASE_SEPOLIA_DARKPOOL_ADDRESS,
            permit2_address: BASE_SEPOLIA_PERMIT2_ADDRESS,
            key: key.clone(),
        }
    }
}
