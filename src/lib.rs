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

#[cfg(feature = "external-match-client")]
pub use external_match_client::*;

#[cfg(feature = "darkpool-client")]
pub(crate) mod renegade_wallet_client;
#[cfg(feature = "darkpool-client")]
pub use renegade_wallet_client::*;

#[cfg(feature = "examples")]
pub mod example_utils;

// -------------
// | Constants |
// -------------

/// The Arbitrum Sepolia relayer base URL
pub(crate) const ARBITRUM_SEPOLIA_RELAYER_BASE_URL: &str =
    "https://arbitrum-sepolia.relayer.renegade.fi";
/// The Arbitrum One relayer base URL
pub(crate) const ARBITRUM_ONE_RELAYER_BASE_URL: &str = "https://arbitrum-one.relayer.renegade.fi";
/// The Base Sepolia relayer base URL
pub(crate) const BASE_SEPOLIA_RELAYER_BASE_URL: &str = "https://base-sepolia.relayer.renegade.fi";
/// The Base mainnet relayer base URL
pub(crate) const BASE_MAINNET_RELAYER_BASE_URL: &str = "https://base-mainnet.relayer.renegade.fi";
