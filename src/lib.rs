//! A Rust SDK for interacting with the Renegade relayer
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(unsafe_code)]
#![deny(clippy::needless_pass_by_ref_mut)]

#[cfg(feature = "external-match-client")]
pub(crate) mod external_match_client;
mod http;
pub mod types;
mod util;

#[cfg(feature = "internal")]
pub use http::*;
pub use util::*;

#[cfg(feature = "external-match-client")]
pub use external_match_client::*;

#[cfg(feature = "darkpool-client")]
pub(crate) mod renegade_wallet_client;
#[cfg(feature = "darkpool-client")]
pub use renegade_wallet_client::*;

#[cfg(feature = "examples")]
pub mod example_utils;

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
/// The Ethereum Sepolia relayer base URL
pub(crate) const ETHEREUM_SEPOLIA_RELAYER_BASE_URL: &str = "https://ethereum-sepolia.relayer.renegade.fi";
/// The Ethereum mainnet relayer base URL
pub(crate) const ETHEREUM_MAINNET_RELAYER_BASE_URL: &str = "https://ethereum-mainnet.relayer.renegade.fi";

// --- Chain IDs --- //

/// The Arbitrum one chain ID
pub const ARBITRUM_ONE_CHAIN_ID: u64 = 42161;
/// The Arbitrum Sepolia chain ID
pub const ARBITRUM_SEPOLIA_CHAIN_ID: u64 = 421614;
/// The Base mainnet chain ID
pub const BASE_MAINNET_CHAIN_ID: u64 = 8453;
/// The Base Sepolia chain ID
pub const BASE_SEPOLIA_CHAIN_ID: u64 = 84532;
/// The Ethereum mainnet chain ID
pub const ETHEREUM_MAINNET_CHAIN_ID: u64 = 1;
/// The Ethereum Sepolia chain ID
pub const ETHEREUM_SEPOLIA_CHAIN_ID: u64 = 11155111;
