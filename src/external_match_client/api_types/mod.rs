//! Types for the external match client
mod fixed_point;
mod order_types;
mod request_response;

pub use fixed_point::*;
pub use order_types::*;
pub use request_response::*;

// ---------------
// | HTTP Routes |
// ---------------

/// Returns the supported token list
pub const GET_SUPPORTED_TOKENS_ROUTE: &str = "/v0/supported-tokens";
/// The route for requesting a quote on an external match
pub const REQUEST_EXTERNAL_QUOTE_ROUTE: &str = "/v0/matching-engine/quote";
/// The route used to assemble an external match quote into a settlement bundle
pub const ASSEMBLE_EXTERNAL_MATCH_ROUTE: &str = "/v0/matching-engine/assemble-external-match";
/// The route used to assemble an external match into a malleable bundle
pub const ASSEMBLE_EXTERNAL_MATCH_MALLEABLE_ROUTE: &str =
    "/v0/matching-engine/assemble-malleable-external-match";
/// The route for requesting an atomic match
pub const REQUEST_EXTERNAL_MATCH_ROUTE: &str = "/v0/matching-engine/request-external-match";
