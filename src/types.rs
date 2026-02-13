//! Re-exports of v1 types for backwards compatibility

#[cfg(feature = "external-match-client")]
pub use crate::external_match_client::api_types::{
    OrderSide,
    v1_types::{ApiExternalQuote, AtomicMatchApiBundle, ExternalOrder, SignedExternalQuote},
};

/// The address used to represent the native asset
pub const NATIVE_ASSET_ADDR: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
