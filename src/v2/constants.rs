//! Shared constants for the v2 Renegade API

// --- Relayer URLs --- //

/// The Arbitrum Sepolia relayer base URL
pub(crate) const ARBITRUM_SEPOLIA_RELAYER_BASE_URL: &str =
    "https://arbitrum-sepolia.relayer.renegade.fi";
/// The Arbitrum One relayer base URL
pub(crate) const ARBITRUM_ONE_RELAYER_BASE_URL: &str = "https://arbitrum-one.relayer.renegade.fi";
/// The Base Sepolia relayer base URL
pub(crate) const BASE_SEPOLIA_RELAYER_BASE_URL: &str = "https://base-sepolia.relayer.renegade.fi";
/// The Base mainnet relayer base URL
pub(crate) const BASE_MAINNET_RELAYER_BASE_URL: &str = "https://base-mainnet.relayer.renegade.fi";

// --- Chain IDs --- //

/// The Arbitrum one chain ID
pub const ARBITRUM_ONE_CHAIN_ID: u64 = 42161;
/// The Arbitrum Sepolia chain ID
pub const ARBITRUM_SEPOLIA_CHAIN_ID: u64 = 421614;
/// The Base mainnet chain ID
pub const BASE_MAINNET_CHAIN_ID: u64 = 8453;
/// The Base Sepolia chain ID
pub const BASE_SEPOLIA_CHAIN_ID: u64 = 84532;
