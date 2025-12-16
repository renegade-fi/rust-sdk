//! Rust SDK for interacting with the v2 Renegade API

#[cfg(feature = "external-match-client")]
pub(crate) mod external_match_client;

pub(crate) mod constants;

#[cfg(feature = "external-match-client")]
pub use external_match_client::*;

pub use constants::*;
