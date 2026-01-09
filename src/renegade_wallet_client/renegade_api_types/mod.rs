//! API types for the Renegade client

mod account;
pub mod admin;
pub mod balances;
pub mod orders;
pub mod request_response;
mod serde_helpers;
pub mod tasks;
pub mod websocket;

// ---------------
// | HTTP Routes |
// ---------------

/// The route for creating an account
pub const CREATE_ACCOUNT_ROUTE: &str = "/v2/account";

/// The route for getting an account's seed CSPRNG states
pub const GET_ACCOUNT_SEEDS_ROUTE: &str = "/v2/account/:account_id/seeds";

/// The route for syncing an account
pub const SYNC_ACCOUNT_ROUTE: &str = "/v2/account/:account_id/sync";

/// The route for getting all balances for an account
pub const GET_BALANCES_ROUTE: &str = "/v2/account/:account_id/balances";

/// The route for getting a balance by mint
pub const GET_BALANCE_BY_MINT_ROUTE: &str = "/v2/account/:account_id/balances/:mint";

/// The route for depositing a balance
pub const DEPOSIT_BALANCE_ROUTE: &str = "/v2/account/:account_id/balances/:mint/deposit";

/// The route for withdrawing a balance
pub const WITHDRAW_BALANCE_ROUTE: &str = "/v2/account/:account_id/balances/:mint/withdraw";

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

/// The route for getting all tasks for an account
pub const GET_TASKS_ROUTE: &str = "/v2/account/:account_id/tasks";

/// The route for getting a task by its ID
pub const GET_TASK_BY_ID_ROUTE: &str = "/v2/account/:account_id/tasks/:task_id";

/// The route for getting all open orders w/ admin metadata
pub const ADMIN_GET_ORDERS_ROUTE: &str = "/v2/relayer-admin/orders";

/// The route for getting a given order w/ admin metadata
pub const ADMIN_GET_ORDER_ROUTE: &str = "/v2/relayer-admin/orders/:order_id";

/// The route for creating an order in a pool (admin)
pub const ADMIN_CREATE_ORDER_IN_POOL_ROUTE: &str = "/v2/relayer-admin/orders/create-order-in-pool";

/// The route for assigning an order to a pool (admin)
pub const ADMIN_ASSIGN_ORDER_TO_POOL_ROUTE: &str =
    "/v2/relayer-admin/orders/:order_id/assign-to-pool";
