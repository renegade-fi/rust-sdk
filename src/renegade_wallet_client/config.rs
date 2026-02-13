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
    BASE_SEPOLIA_CHAIN_ID, BASE_SEPOLIA_RELAYER_BASE_URL, ETHEREUM_SEPOLIA_CHAIN_ID,
    ETHEREUM_SEPOLIA_RELAYER_BASE_URL,
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
    address!("0xC5D1B8096BbdEC83Bc6049e42822c7483BBA6500");
/// The darkpool address on Arbitrum Sepolia
pub(crate) const ARBITRUM_SEPOLIA_DARKPOOL_ADDRESS: Address =
    address!("0x57dF3a4449aaBf72f61e4A5DFe83d4A45DcC8537");
/// The darkpool address on Base Mainnet
pub(crate) const BASE_MAINNET_DARKPOOL_ADDRESS: Address =
    address!("0x15d7CF277BE6463F153Dd0d4d73F92Ad65e6348C");
/// The darkpool address on Base Sepolia
pub(crate) const BASE_SEPOLIA_DARKPOOL_ADDRESS: Address =
    address!("0xDE9BfD62B2187d4c14FBcC7D869920d34e4DB3Da");
/// The darkpool address on Ethereum Sepolia
pub(crate) const ETHEREUM_SEPOLIA_DARKPOOL_ADDRESS: Address =
    address!("0x45537c28F245645CC1E7F7258FCC18A189CE16e3");

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

// --- Executor Addresses --- //

/// The executor address on Arbitrum One
pub(crate) const ARBITRUM_ONE_EXECUTOR_ADDRESS: Address =
    address!("0x336A6b8AE5589d40ba4391020649E268E8323CA1");
/// The executor address on Arbitrum Sepolia
pub(crate) const ARBITRUM_SEPOLIA_EXECUTOR_ADDRESS: Address =
    address!("0x9094314D60e3eF5fC73df548A3dD7b1Cd9798729");
/// The executor address on Base Mainnet
pub(crate) const BASE_MAINNET_EXECUTOR_ADDRESS: Address =
    address!("0x1b5A1833d8566FACb138aa6BF1cd040f572B1D56");
/// The executor address on Base Sepolia
pub(crate) const BASE_SEPOLIA_EXECUTOR_ADDRESS: Address =
    address!("0x5E2ca57B7F09Cf3DAca07c67CC65e1BfbDf346b0");
/// The executor address on Ethereum Sepolia
pub(crate) const ETHEREUM_SEPOLIA_EXECUTOR_ADDRESS: Address =
    address!("0x92467D2FF278383187f0aB04F8511EF45c31b723");

// --- Relayer Fee Recipient Addresses --- //

/// The relayer fee recipient address on Arbitrum One
pub(crate) const ARBITRUM_ONE_RELAYER_FEE_RECIPIENT: Address =
    address!("0x0000000000000000000000000000000000000000");
/// The relayer fee recipient address on Arbitrum Sepolia
pub(crate) const ARBITRUM_SEPOLIA_RELAYER_FEE_RECIPIENT: Address =
    address!("0xb0c0d3e8ebc39df5799d9c98d65dacf8637deba1");
/// The relayer fee recipient address on Base Mainnet
pub(crate) const BASE_MAINNET_RELAYER_FEE_RECIPIENT: Address =
    address!("0x0000000000000000000000000000000000000000");
/// The relayer fee recipient address on Base Sepolia
pub(crate) const BASE_SEPOLIA_RELAYER_FEE_RECIPIENT: Address =
    address!("0xa125ecd644591348d08243d8821120c6d7d3a077");
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
    pub fn new_ethereum_sepolia_admin(key: &PrivateKeySigner, admin_hmac_key: HmacKey) -> Self {
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
