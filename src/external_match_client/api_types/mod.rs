//! Types for the external match client
pub mod exchange_metadata;
mod fixed_point;
mod malleable_match;
pub mod order_book;
mod order_types;
mod request_response;
mod serde_helpers;
pub mod token;

pub use fixed_point::*;
pub use order_types::*;
pub use request_response::*;

// ---------------
// | HTTP Routes |
// ---------------

/// Returns the supported token list
pub const GET_SUPPORTED_TOKENS_ROUTE: &str = "/v0/supported-tokens";

/// Get token prices for all supported tokens
pub const GET_TOKEN_PRICES_ROUTE: &str = "/v0/token-prices";

/// Returns metadata about the Renegade exchange
pub const GET_EXCHANGE_METADATA_ROUTE: &str = "/v0/exchange-metadata";

/// The order book depth format string
pub const ORDER_BOOK_DEPTH_ROUTE: &str = "/v0/order_book/depth";

/// The route for requesting a quote on an external match
pub const GET_QUOTE_ROUTE: &str = "/v2/external-matches/get-quote";

/// The route for assembling an external match bundle
pub const ASSEMBLE_MATCH_BUNDLE_ROUTE: &str = "/v2/external-matches/assemble-match-bundle";
