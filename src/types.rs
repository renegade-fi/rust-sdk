//! Types for the Renegade SDK

// We re-export these types here because they were previously re-exported here
// from `renegade` dependencies, and we don't want to break existing code that
// uses them.
pub use crate::external_match_client::api_types::{
    ApiExternalQuote, AtomicMatchApiBundle, ExternalOrder, OrderSide, SignedExternalQuote,
};

/// The address used to represent the native asset
pub const NATIVE_ASSET_ADDR: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
