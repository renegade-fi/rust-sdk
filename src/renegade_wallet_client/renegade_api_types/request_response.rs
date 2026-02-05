//! Top-level API request / response types

use alloy::primitives::Address;
use renegade_circuit_types::Amount;
use renegade_constants::Scalar;
use renegade_types_core::HmacKey;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::renegade_api_types::{
    account::{ApiAccount, ApiPoseidonCSPRNG},
    admin::{ApiAdminOrder, ApiAdminOrderCore},
    balances::{ApiBalance, ApiDepositPermit, ApiSchnorrPublicKey},
    orders::{ApiOrder, ApiOrderCore, OrderAuth},
    tasks::ApiTask,
};

use super::serde_helpers::*;

// ---------------
// | Account API |
// ---------------

/// A request to create an account
#[derive(Debug, Serialize)]
pub struct CreateAccountRequest {
    /// The account ID
    pub account_id: Uuid,
    /// The address of the account owner
    pub address: Address,
    /// The master view seed
    #[serde(with = "scalar_string_serde")]
    pub master_view_seed: Scalar,
    /// The HMAC key for API authentication
    #[serde(serialize_with = "serialize_hmac_key")]
    pub auth_hmac_key: HmacKey,
}

/// A response containing an account
#[derive(Debug, Deserialize)]
pub struct GetAccountResponse {
    /// The account
    pub account: ApiAccount,
}

/// A response containing the current states of an account's
/// seed CSPRNGs
#[derive(Debug, Deserialize)]
pub struct GetAccountSeedsResponse {
    /// The current state of the recovery stream seeds CSPRNG
    pub recovery_seed_csprng: ApiPoseidonCSPRNG,
    /// The current state of the share stream seeds CSPRNG
    pub share_seed_csprng: ApiPoseidonCSPRNG,
}

/// The query parameters used when syncing an account
#[derive(Debug, Default, Serialize)]
pub struct SyncAccountQueryParameters {
    /// Whether to block on the completion of the account sync task before
    /// receiving a response
    pub non_blocking: Option<bool>,
}

/// A request to sync an account
#[derive(Debug, Serialize)]
pub struct SyncAccountRequest {
    /// The account ID
    pub account_id: Uuid,
    /// The master view seed
    #[serde(with = "scalar_string_serde")]
    pub master_view_seed: Scalar,
    /// The HMAC key for API authentication
    #[serde(serialize_with = "serialize_hmac_key")]
    pub auth_hmac_key: HmacKey,
}

/// The response received after syncing an account
#[derive(Debug, Deserialize)]
pub struct SyncAccountResponse {
    /// The ID of the account sync task spawned in the relayer
    pub task_id: Uuid,
    /// Whether the account sync task has completed
    pub completed: bool,
}

// # === Orders === #

/// The query parameters used when fetching all account orders
#[derive(Debug, Default, Serialize)]
pub struct GetOrdersQueryParameters {
    /// Whether to include historic (inactive) orders in the response
    pub include_historic_orders: Option<bool>,
    /// The number of orders to return per page
    pub page_size: Option<usize>,
    /// The page token to use for pagination
    pub page_token: Option<usize>,
}

/// A response containing a page of orders
#[derive(Debug, Deserialize)]
pub struct GetOrdersResponse {
    /// The orders
    pub orders: Vec<ApiOrder>,
    /// The next page token to use for pagination, if more orders are available
    pub next_page_token: Option<usize>,
}

/// A response containing a single order
#[derive(Debug, Deserialize)]
pub struct GetOrderByIdResponse {
    /// The order
    pub order: ApiOrder,
}

/// The query parameters used when creating an order
#[derive(Debug, Default, Serialize)]
pub struct CreateOrderQueryParameters {
    /// Whether to block on the completion of the order creation task before
    /// receiving a response
    pub non_blocking: Option<bool>,
}

/// A request to create an order
#[derive(Debug, Serialize)]
pub struct CreateOrderRequest {
    /// The order to create
    pub order: ApiOrderCore,
    /// The authorization of the order creation
    pub auth: OrderAuth,
    /// Whether to precompute a cancellation proof for the order
    pub precompute_cancellation_proof: bool,
}

/// The response received after creating an order
#[derive(Debug, Deserialize)]
pub struct CreateOrderResponse {
    /// The ID of the order creation task spawned in the relayer
    pub task_id: Uuid,
    /// Whether the order creation task has completed
    pub completed: bool,
}

/// A request to update an order
#[derive(Debug, Serialize)]
pub struct UpdateOrderRequest {
    /// The updated order
    pub order: ApiOrderCore,
}

/// The response received after updating an order
#[derive(Debug, Deserialize)]
pub struct UpdateOrderResponse {
    /// The updated order
    pub order: ApiOrder,
}

/// The query parameters used when cancelling an order
#[derive(Debug, Default, Serialize)]
pub struct CancelOrderQueryParameters {
    /// Whether to block on the completion of the order cancellation task before
    /// receiving a response
    pub non_blocking: Option<bool>,
}

/// A request to cancel an order
#[derive(Debug, Default, Serialize)]
pub struct CancelOrderRequest {
    /// The signature over the order's nullifier which authorizes its
    /// cancellation
    #[serde(serialize_with = "serialize_bytes_b64")]
    pub signature: Vec<u8>,
}

/// The response received after cancelling an order
#[derive(Debug, Deserialize)]
pub struct CancelOrderResponse {
    /// The ID of the order cancellation task spawned in the relayer
    pub task_id: Uuid,
    /// The order that was cancelled
    pub order: ApiOrder,
    /// Whether the order cancellation task has completed
    pub completed: bool,
}

// # === Balances === #

/// A response containing all balances for an account
#[derive(Debug, Deserialize)]
pub struct GetBalancesResponse {
    /// The balances
    pub balances: Vec<ApiBalance>,
}

/// A response containing a single balance
#[derive(Debug, Deserialize)]
pub struct GetBalanceByMintResponse {
    /// The balance
    pub balance: ApiBalance,
}

/// The query parameters used when depositing a balance
#[derive(Debug, Default, Serialize)]
pub struct DepositBalanceQueryParameters {
    /// Whether to block on the completion of the deposit task before
    /// receiving a response
    pub non_blocking: Option<bool>,
}

/// A request to deposit a balance
#[derive(Debug, Serialize)]
pub struct DepositBalanceRequest {
    /// The address from which to transfer funds into the darkpool for the
    /// deposit
    pub from_address: Address,
    /// The amount of the token to deposit
    #[serde(with = "amount_string_serde")]
    pub amount: Amount,
    /// The authority public key to use, in case a new balance needs to be
    /// created
    pub authority: ApiSchnorrPublicKey,
    /// The permit authorizing the deposit
    pub permit: ApiDepositPermit,
}

/// The response received after depositing a balance
#[derive(Debug, Deserialize)]
pub struct DepositBalanceResponse {
    /// The ID of the deposit task spawned in the relayer
    pub task_id: Uuid,
    /// Whether the deposit task has completed
    pub completed: bool,
}

/// The query parameters used when withdrawing from a balance
#[derive(Debug, Default, Serialize)]
pub struct WithdrawBalanceQueryParameters {
    /// Whether to block on the completion of the withdrawal task before
    /// receiving a response
    pub non_blocking: Option<bool>,
}

/// A request to withdraw from a balance
#[derive(Debug, Serialize)]
pub struct WithdrawBalanceRequest {
    /// The amount of the token to withdraw
    #[serde(with = "amount_string_serde")]
    pub amount: Amount,
    /// The signature over the balance commitment which authorizes the
    /// withdrawal
    #[serde(serialize_with = "serialize_bytes_b64")]
    pub signature: Vec<u8>,
}

/// The response received after withdrawing from a balance
#[derive(Debug, Deserialize)]
pub struct WithdrawBalanceResponse {
    /// The ID of the withdrawal task spawned in the relayer
    pub task_id: Uuid,
    /// Whether the withdrawal task has completed
    pub completed: bool,
}

// # === Tasks === #

/// The query parameters used when fetching all account tasks
#[derive(Debug, Default, Serialize)]
pub struct GetTasksQueryParameters {
    /// Whether to include historic tasks in the response
    pub include_historic_tasks: Option<bool>,
    /// The number of tasks to return per page
    pub page_size: Option<usize>,
    /// The page token to use for pagination
    pub page_token: Option<usize>,
}

/// A response containing a page of tasks
#[derive(Debug, Deserialize)]
pub struct GetTasksResponse {
    /// The tasks
    pub tasks: Vec<ApiTask>,
    /// The next page token to use for pagination, if more tasks are available
    pub next_page_token: Option<usize>,
}

/// A response containing a single task
#[derive(Debug, Deserialize)]
pub struct GetTaskByIdResponse {
    /// The task
    pub task: ApiTask,
}

// -------------
// | Admin API |
// -------------

/// The query parameters used when fetching all open orders managed by the
/// relayer
#[derive(Debug, Default, Serialize)]
pub struct GetOpenOrdersAdminQueryParameters {
    /// The matching pool from which to fetch orders
    pub matching_pool: Option<String>,
    /// The number of orders to return per page
    pub page_size: Option<usize>,
    /// The page token to use for pagination
    pub page_token: Option<usize>,
}

/// A response containing a page of open orders w/ admin-level metadata
#[derive(Debug, Deserialize)]
pub struct GetOpenOrdersAdminResponse {
    /// The orders
    pub orders: Vec<ApiAdminOrder>,
    /// The next page token to use for pagination, if more orders are available
    pub next_page_token: Option<usize>,
}

/// The query parameters used when fetching all orders for a given account
/// (admin)
#[derive(Debug, Default, Serialize)]
pub struct GetAccountOrdersAdminQueryParameters {
    /// The number of orders to return per page
    pub page_size: Option<usize>,
    /// The page token to use for pagination
    pub page_token: Option<usize>,
}

/// A response containing a page of orders for a given account w/ admin-level
/// metadata
#[derive(Debug, Deserialize)]
pub struct GetAccountOrdersAdminResponse {
    /// The orders
    pub orders: Vec<ApiAdminOrder>,
    /// The next page token to use for pagination, if more orders are available
    pub next_page_token: Option<usize>,
}

/// A response containing a single order w/ admin-level metadata
#[derive(Debug, Deserialize)]
pub struct GetOrderAdminResponse {
    /// The order
    pub order: ApiAdminOrder,
}

/// The query parameters used when creating an order in a pool (admin)
#[derive(Debug, Default, Serialize)]
pub struct AdminCreateOrderInPoolQueryParameters {
    /// Whether to block on the completion of the order creation task before
    /// receiving a response
    pub non_blocking: Option<bool>,
}

/// A request to create an order in a pool (admin)
#[derive(Debug, Serialize)]
pub struct AdminCreateOrderInPoolRequest {
    /// The order to create
    pub order: ApiAdminOrderCore,
    /// The authorization of the order creation
    pub auth: OrderAuth,
    /// Whether to precompute a cancellation proof for the order
    pub precompute_cancellation_proof: bool,
}

/// The response received after creating an order in a pool (admin)
#[derive(Debug, Deserialize)]
pub struct AdminCreateOrderInPoolResponse {
    /// The ID of the order creation task spawned in the relayer
    pub task_id: Uuid,
    /// Whether the order creation task has completed
    pub completed: bool,
}

/// A request to assign an order to a pool (admin)
#[derive(Debug, Serialize)]
pub struct AdminAssignOrderToPoolRequest {
    /// The matching pool to assign the order to
    pub matching_pool: String,
}

/// The response received after assigning an order to a pool (admin)
#[derive(Debug, Deserialize)]
pub struct AdminAssignOrderToPoolResponse {
    /// The order with updated pool assignment
    pub order: ApiAdminOrder,
}

/// A request to create a matching pool (admin)
#[derive(Debug, Serialize)]
pub struct AdminCreateMatchingPoolRequest {
    /// The name of the matching pool to create
    pub matching_pool: String,
}

/// The response received after checking if a task queue is paused (admin)
#[derive(Debug, Deserialize)]
pub struct AdminTaskQueuePausedResponse {
    /// Whether the task queue is paused
    pub paused: bool,
}
