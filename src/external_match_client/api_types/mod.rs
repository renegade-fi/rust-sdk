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
#[deprecated(since = "2.0.0", note = "Use GET_QUOTE_ROUTE instead")]
pub const REQUEST_EXTERNAL_QUOTE_ROUTE: &str = GET_QUOTE_ROUTE;
/// The route for requesting a quote on an external match
pub const GET_QUOTE_ROUTE: &str = "/v2/external-matches/get-quote";

/// The route used to assemble an external match quote into a settlement bundle
pub const ASSEMBLE_EXTERNAL_MATCH_ROUTE: &str = "/v0/matching-engine/assemble-external-match";

/// The route used to assemble an external match into a malleable bundle
pub const ASSEMBLE_EXTERNAL_MATCH_MALLEABLE_ROUTE: &str =
    "/v0/matching-engine/assemble-malleable-external-match";

/// The route for requesting an atomic match
pub const REQUEST_EXTERNAL_MATCH_ROUTE: &str = "/v0/matching-engine/request-external-match";

/// The route for requesting a malleable match
pub const REQUEST_MALLEABLE_EXTERNAL_MATCH_ROUTE: &str =
    "/v0/matching-engine/request-malleable-external-match";
