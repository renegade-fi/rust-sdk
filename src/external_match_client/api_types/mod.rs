//! Types for the external match client
pub mod exchange_metadata;
mod fixed_point;
mod malleable_match;
pub mod markets;
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

/// The route for fetching all tradable markets
pub const GET_MARKETS_ROUTE: &str = "/v2/markets";

/// The route for fetching the depth of all marketse
pub const GET_MARKETS_DEPTH_ROUTE: &str = "/v2/markets/depth";

/// The route for fetching the depth of a specific market
pub const GET_MARKET_DEPTH_BY_MINT_ROUTE: &str = "/v2/markets/:mint/depth";

/// Returns metadata about the Renegade exchange
pub const GET_EXCHANGE_METADATA_ROUTE: &str = "/v2/metadata/exchange";

/// The route for requesting a quote on an external match
pub const GET_QUOTE_ROUTE: &str = "/v2/external-matches/get-quote";

/// The route for assembling an external match bundle
pub const ASSEMBLE_MATCH_BUNDLE_ROUTE: &str = "/v2/external-matches/assemble-match-bundle";
