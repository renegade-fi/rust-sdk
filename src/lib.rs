//! A Rust SDK for interacting with the Renegade relayer
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(unsafe_code)]
#![deny(clippy::needless_pass_by_ref_mut)]

#[cfg(feature = "darkpool-client")]
mod darkpool_client;
#[cfg(feature = "external-match-client")]
mod external_match_client;
mod http;
pub mod types;
mod util;

#[cfg(feature = "darkpool-client")]
pub use darkpool_client::DarkpoolClient;
#[cfg(feature = "external-match-client")]
pub use external_match_client::*;
