//! A Rust SDK for interacting with the Renegade relayer
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(unsafe_code)]
#![deny(clippy::needless_pass_by_ref_mut)]

mod external_match_client;
mod http;
pub mod types;

pub use external_match_client::*;
