//! API types for the Renegade client

mod account;
pub mod request_response;
mod serde_helpers;

// ---------------
// | HTTP Routes |
// ---------------

/// The route for creating an account
pub const CREATE_ACCOUNT_ROUTE: &str = "/v2/account";

/// The route for getting an account's seed CSPRNG states
pub const GET_ACCOUNT_SEEDS_ROUTE: &str = "/v2/account/:account_id/seeds";
