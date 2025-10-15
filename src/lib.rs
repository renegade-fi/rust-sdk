//! A Rust SDK for interacting with the Renegade relayer
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(unsafe_code)]
#![deny(clippy::needless_pass_by_ref_mut)]
#![feature(let_chains)]

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
