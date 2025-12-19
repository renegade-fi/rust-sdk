//! API types for the Renegade client

use uuid::Uuid;

mod account;
pub mod orders;
pub mod request_response;
mod serde_helpers;

// ----------------
// | Type Aliases |
// ----------------

/// A type alias for a task identifier
pub type TaskIdentifier = Uuid;

// ---------------
// | HTTP Routes |
// ---------------

/// The route for creating an account
pub const CREATE_ACCOUNT_ROUTE: &str = "/v2/account";

/// The route for getting an account's seed CSPRNG states
pub const GET_ACCOUNT_SEEDS_ROUTE: &str = "/v2/account/:account_id/seeds";

/// The route for getting all orders for an account
pub const GET_ORDERS_ROUTE: &str = "/v2/account/:account_id/orders";

/// The route for getting an order by its ID
pub const GET_ORDER_BY_ID_ROUTE: &str = "/v2/account/:account_id/orders/:order_id";

/// The route for creating an order
pub const CREATE_ORDER_ROUTE: &str = "/v2/account/:account_id/orders";

/// The route for updating an order
pub const UPDATE_ORDER_ROUTE: &str = "/v2/account/:account_id/orders/:order_id/update";

/// The route for cancelling an order
pub const CANCEL_ORDER_ROUTE: &str = "/v2/account/:account_id/orders/:order_id/cancel";
