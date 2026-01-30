//! Config setup for the renegade wallet client

// -------------
// | Constants |
// -------------

use alloy::{
    primitives::{Address, address},
    signers::local::PrivateKeySigner,
};
use renegade_types_core::HmacKey;

use crate::{
    ARBITRUM_ONE_CHAIN_ID, ARBITRUM_ONE_RELAYER_BASE_URL, ARBITRUM_SEPOLIA_CHAIN_ID,
    ARBITRUM_SEPOLIA_RELAYER_BASE_URL, BASE_MAINNET_CHAIN_ID, BASE_MAINNET_RELAYER_BASE_URL,
    BASE_SEPOLIA_CHAIN_ID, BASE_SEPOLIA_RELAYER_BASE_URL,
    ETHEREUM_SEPOLIA_CHAIN_ID, ETHEREUM_SEPOLIA_RELAYER_BASE_URL,
    //ETHEREUM_MAINNET_CHAIN_ID, ETHEREUM_MAINNET_RELAYER_BASE_URL,
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
/// The darkpool address on Ethereum Sepolia
pub(crate) const ETHEREUM_SEPOLIA_DARKPOOL_ADDRESS: Address =
    address!("0x12319dd18C6C10029E60f59862028fe939A1c6e1");
/// The darkpool address on Ethereum Mainnet
//pub(crate) const ETHEREUM_MAINNET_DARKPOOL_ADDRESS: Address =
    //address!("0x0000000000000000000000000000000000000000"); // not deployed yet

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
/// The permit2 address on Ethereum Sepolia
pub(crate) const ETHEREUM_SEPOLIA_PERMIT2_ADDRESS: Address =
    address!("0x000000000022D473030F116dDEE9F6B43aC78BA3");
///// The permit2 address on Ethereum Mainnet
//pub(crate) const ETHEREUM_MAINNET_PERMIT2_ADDRESS: Address =
    //address!("0x000000000022D473030F116dDEE9F6B43aC78BA3");

// --- Executor Addresses --- //

/// The executor address on Arbitrum One
pub(crate) const ARBITRUM_ONE_EXECUTOR_ADDRESS: Address =
    address!("0x0000000000000000000000000000000000000000");
/// The executor address on Arbitrum Sepolia
pub(crate) const ARBITRUM_SEPOLIA_EXECUTOR_ADDRESS: Address =
    address!("0x0000000000000000000000000000000000000000");
/// The executor address on Base Mainnet
pub(crate) const BASE_MAINNET_EXECUTOR_ADDRESS: Address =
    address!("0x0000000000000000000000000000000000000000");
/// The executor address on Base Sepolia
pub(crate) const BASE_SEPOLIA_EXECUTOR_ADDRESS: Address =
    address!("0x0000000000000000000000000000000000000000");
///// The executor address on Ethereum Mainnet
//pub(crate) const ETHEREUM_MAINNET_EXECUTOR_ADDRESS: Address =
    //address!("0x0000000000000000000000000000000000000000");
/// The executor address on Ethereum Sepolia
pub(crate) const ETHEREUM_SEPOLIA_EXECUTOR_ADDRESS: Address =
    address!("0x0000000000000000000000000000000000000000");

// --- Relayer Fee Recipient Addresses --- //

/// The relayer fee recipient address on Arbitrum One
pub(crate) const ARBITRUM_ONE_RELAYER_FEE_RECIPIENT: Address =
    address!("0x0000000000000000000000000000000000000000");
/// The relayer fee recipient address on Arbitrum Sepolia
pub(crate) const ARBITRUM_SEPOLIA_RELAYER_FEE_RECIPIENT: Address =
    address!("0x0000000000000000000000000000000000000000");
/// The relayer fee recipient address on Base Mainnet
pub(crate) const BASE_MAINNET_RELAYER_FEE_RECIPIENT: Address =
    address!("0x0000000000000000000000000000000000000000");
/// The relayer fee recipient address on Base Sepolia
pub(crate) const BASE_SEPOLIA_RELAYER_FEE_RECIPIENT: Address =
    address!("0x0000000000000000000000000000000000000000");
///// The relayer fee recipient address on Ethereum Mainnet
//pub(crate) const ETHEREUM_MAINNET_RELAYER_FEE_RECIPIENT: Address =
    //address!("0x0000000000000000000000000000000000000000");
/// The relayer fee recipient address on Ethereum Sepolia
pub(crate) const ETHEREUM_SEPOLIA_RELAYER_FEE_RECIPIENT: Address =
    address!("0x0000000000000000000000000000000000000000");

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
    /// The relayer's executor address
    pub executor_address: Address,
    /// The relayer's fee recipient address
    pub relayer_fee_recipient: Address,
    /// The private key from which to derive the wallet
    pub key: PrivateKeySigner,
    /// The HMAC key used to authenticate admin API actions
    pub admin_hmac_key: Option<HmacKey>,
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
            executor_address: ARBITRUM_ONE_EXECUTOR_ADDRESS,
            relayer_fee_recipient: ARBITRUM_ONE_RELAYER_FEE_RECIPIENT,
            key: key.clone(),
            admin_hmac_key: None,
        }
    }

    /// Create a new admin client config for Arbitrum One
    pub fn new_arbitrum_one_admin(key: &PrivateKeySigner, admin_hmac_key: HmacKey) -> Self {
        Self {
            relayer_base_url: ARBITRUM_ONE_RELAYER_BASE_URL.to_string(),
            historical_state_base_url: MAINNET_HISTORICAL_STATE_BASE_URL.to_string(),
            chain_id: ARBITRUM_ONE_CHAIN_ID,
            darkpool_address: ARBITRUM_ONE_DARKPOOL_ADDRESS,
            permit2_address: ARBITRUM_ONE_PERMIT2_ADDRESS,
            executor_address: ARBITRUM_ONE_EXECUTOR_ADDRESS,
            relayer_fee_recipient: ARBITRUM_ONE_RELAYER_FEE_RECIPIENT,
            key: key.clone(),
            admin_hmac_key: Some(admin_hmac_key),
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
            executor_address: ARBITRUM_SEPOLIA_EXECUTOR_ADDRESS,
            relayer_fee_recipient: ARBITRUM_SEPOLIA_RELAYER_FEE_RECIPIENT,
            key: key.clone(),
            admin_hmac_key: None,
        }
    }

    /// Create a new admin client config for Arbitrum Sepolia
    pub fn new_arbitrum_sepolia_admin(key: &PrivateKeySigner, admin_hmac_key: HmacKey) -> Self {
        Self {
            relayer_base_url: ARBITRUM_SEPOLIA_RELAYER_BASE_URL.to_string(),
            historical_state_base_url: TESTNET_HISTORICAL_STATE_BASE_URL.to_string(),
            chain_id: ARBITRUM_SEPOLIA_CHAIN_ID,
            darkpool_address: ARBITRUM_SEPOLIA_DARKPOOL_ADDRESS,
            permit2_address: ARBITRUM_SEPOLIA_PERMIT2_ADDRESS,
            executor_address: ARBITRUM_SEPOLIA_EXECUTOR_ADDRESS,
            relayer_fee_recipient: ARBITRUM_SEPOLIA_RELAYER_FEE_RECIPIENT,
            key: key.clone(),
            admin_hmac_key: Some(admin_hmac_key),
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
            executor_address: BASE_MAINNET_EXECUTOR_ADDRESS,
            relayer_fee_recipient: BASE_MAINNET_RELAYER_FEE_RECIPIENT,
            key: key.clone(),
            admin_hmac_key: None,
        }
    }

    /// Create a new admin client config for Base Mainnet
    pub fn new_base_mainnet_admin(key: &PrivateKeySigner, admin_hmac_key: HmacKey) -> Self {
        Self {
            relayer_base_url: BASE_MAINNET_RELAYER_BASE_URL.to_string(),
            historical_state_base_url: MAINNET_HISTORICAL_STATE_BASE_URL.to_string(),
            chain_id: BASE_MAINNET_CHAIN_ID,
            darkpool_address: BASE_MAINNET_DARKPOOL_ADDRESS,
            permit2_address: BASE_MAINNET_PERMIT2_ADDRESS,
            executor_address: BASE_MAINNET_EXECUTOR_ADDRESS,
            relayer_fee_recipient: BASE_MAINNET_RELAYER_FEE_RECIPIENT,
            key: key.clone(),
            admin_hmac_key: Some(admin_hmac_key),
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
            executor_address: BASE_SEPOLIA_EXECUTOR_ADDRESS,
            relayer_fee_recipient: BASE_SEPOLIA_RELAYER_FEE_RECIPIENT,
            key: key.clone(),
            admin_hmac_key: None,
        }
    }

    /// Create a new admin client config for Base Sepolia
    pub fn new_base_sepolia_admin(key: &PrivateKeySigner, admin_hmac_key: HmacKey) -> Self {
        Self {
            relayer_base_url: BASE_SEPOLIA_RELAYER_BASE_URL.to_string(),
            historical_state_base_url: TESTNET_HISTORICAL_STATE_BASE_URL.to_string(),
            chain_id: BASE_SEPOLIA_CHAIN_ID,
            darkpool_address: BASE_SEPOLIA_DARKPOOL_ADDRESS,
            permit2_address: BASE_SEPOLIA_PERMIT2_ADDRESS,
            executor_address: BASE_SEPOLIA_EXECUTOR_ADDRESS,
            relayer_fee_recipient: BASE_SEPOLIA_RELAYER_FEE_RECIPIENT,
            key: key.clone(),
            admin_hmac_key: Some(admin_hmac_key),
        }
    }

    ///// Create a new client config for Ethereum Mainnet
    //pub fn new_ethereum_mainnet(key: &PrivateKeySigner) -> Self {
        //Self {
            //relayer_base_url: ETHEREUM_MAINNET_RELAYER_BASE_URL.to_string(),
            //historical_state_base_url: MAINNET_HISTORICAL_STATE_BASE_URL.to_string(),
            //chain_id: ETHEREUM_MAINNET_CHAIN_ID,
            //darkpool_address: ETHEREUM_MAINNET_DARKPOOL_ADDRESS,
            //permit2_address: ETHEREUM_MAINNET_PERMIT2_ADDRESS,
            //key: key.clone(),
        //}
    //}

    /// Create a new client config for Ethereum Sepolia
    pub fn new_ethereum_sepolia(key: &PrivateKeySigner) -> Self {
        Self {
            relayer_base_url: ETHEREUM_SEPOLIA_RELAYER_BASE_URL.to_string(),
            historical_state_base_url: TESTNET_HISTORICAL_STATE_BASE_URL.to_string(),
            chain_id: ETHEREUM_SEPOLIA_CHAIN_ID,
            darkpool_address: ETHEREUM_SEPOLIA_DARKPOOL_ADDRESS,
            permit2_address: ETHEREUM_SEPOLIA_PERMIT2_ADDRESS,
            executor_address: ETHEREUM_SEPOLIA_EXECUTOR_ADDRESS,
            relayer_fee_recipient: ETHEREUM_SEPOLIA_RELAYER_FEE_RECIPIENT,
            key: key.clone(),
            admin_hmac_key: None,
        }
    }

    /// Create a new admin client config for Ethereum Sepolia
    pub fn new_ethereum_sepolia_admin(key: &PrivateKeySigner, admin_hmac_key: &HmacKey) -> Self {
        Self {
            relayer_base_url: ETHEREUM_SEPOLIA_RELAYER_BASE_URL.to_string(),
            historical_state_base_url: TESTNET_HISTORICAL_STATE_BASE_URL.to_string(),
            chain_id: ETHEREUM_SEPOLIA_CHAIN_ID,
            darkpool_address: ETHEREUM_SEPOLIA_DARKPOOL_ADDRESS,
            permit2_address: ETHEREUM_SEPOLIA_PERMIT2_ADDRESS,
            executor_address: ETHEREUM_SEPOLIA_EXECUTOR_ADDRESS,
            relayer_fee_recipient: ETHEREUM_SEPOLIA_RELAYER_FEE_RECIPIENT,
            key: key.clone(),
            admin_hmac_key: Some(admin_hmac_key),
        }
    }
}
