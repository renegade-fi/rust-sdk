//! Utility functions for the renegade-sdk
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    ARBITRUM_ONE_CHAIN_ID,
    ARBITRUM_SEPOLIA_CHAIN_ID,
    BASE_MAINNET_CHAIN_ID,
    BASE_SEPOLIA_CHAIN_ID,
    // ETHEREUM_MAINNET_CHAIN_ID,
    ETHEREUM_SEPOLIA_CHAIN_ID,
};

// -----------
// | Helpers |
// -----------

/// Returns the current unix timestamp in milliseconds, represented as u64
pub fn get_current_time_millis() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("negative timestamp").as_millis() as u64
}

/// Returns the environment-agnostic name of the chain with the given ID
pub fn get_env_agnostic_chain(chain_id: u64) -> String {
    match chain_id {
        ARBITRUM_ONE_CHAIN_ID | ARBITRUM_SEPOLIA_CHAIN_ID => "arbitrum".to_string(),
        BASE_MAINNET_CHAIN_ID | BASE_SEPOLIA_CHAIN_ID => "base".to_string(),
        // ETHEREUM_MAINNET_CHAIN_ID |
        ETHEREUM_SEPOLIA_CHAIN_ID => "ethereum".to_string(),
        _ => panic!("Unsupported chain ID: {chain_id}"),
    }
}
