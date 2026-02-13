//! Admin action to place an order in a specific matching pool

use std::str::FromStr;

use alloy::primitives::Address;
use renegade_circuit_types::{Amount, fixed_point::FixedPoint};
use renegade_external_api::{
    http::{
        admin::ADMIN_CREATE_ORDER_IN_POOL_ROUTE,
        order::{CreateOrderInPoolRequest, CreateOrderResponse},
    },
    types::{ApiIntent, ApiOrderCore, OrderType},
};
use uuid::Uuid;

use crate::{
    RenegadeClientError,
    actions::{NON_BLOCKING_PARAM, construct_http_path},
    client::RenegadeClient,
    utils::unwrap_field,
    websocket::{DEFAULT_TASK_TIMEOUT, TaskWaiter},
};

// --- Public Actions --- //
impl RenegadeClient {
    /// Create a new admin order builder with the client's account address as
    /// the owner
    pub fn new_admin_order_builder(&self) -> AdminOrderBuilder {
        AdminOrderBuilder::new(self.get_account_address())
    }

    /// Places an order in a specific matching pool via the admin API.
    /// Waits for the order creation task to complete before returning.
    ///
    /// This is an admin action that requires the client to be configured with
    /// an admin HMAC key.
    pub async fn admin_place_order_in_pool(
        &self,
        built_order: BuiltAdminOrder,
    ) -> Result<(), RenegadeClientError> {
        let admin_client = self.get_admin_client()?;

        let request = self.build_admin_create_order_request(built_order).await?;

        let path = self.build_admin_create_order_request_path(false)?;

        admin_client.post::<_, CreateOrderResponse>(&path, request).await?;

        Ok(())
    }

    /// Enqueues an order placement task in a specific matching pool via the
    /// admin API. Returns the expected order to be created, and a `TaskWaiter`
    /// that can be used to await task completion.
    ///
    /// This is an admin action that requires the client to be configured with
    /// an admin HMAC key.
    pub async fn enqueue_admin_order_placement_in_pool(
        &self,
        built_order: BuiltAdminOrder,
    ) -> Result<TaskWaiter, RenegadeClientError> {
        let admin_client = self.get_admin_client()?;

        let request = self.build_admin_create_order_request(built_order).await?;

        let path = self.build_admin_create_order_request_path(true)?;

        let CreateOrderResponse { task_id, .. } = admin_client.post(&path, request).await?;

        // Create a task waiter for the task
        let task_waiter = self.watch_task(task_id, DEFAULT_TASK_TIMEOUT).await?;

        Ok(task_waiter)
    }
}

// --- Private Helpers --- //
impl RenegadeClient {
    /// Builds the admin order creation request from the given built admin order
    async fn build_admin_create_order_request(
        &self,
        built_order: BuiltAdminOrder,
    ) -> Result<CreateOrderInPoolRequest, RenegadeClientError> {
        let auth = self.build_order_auth(&built_order.order).await?;

        Ok(CreateOrderInPoolRequest {
            order: built_order.order,
            auth,
            matching_pool: built_order.matching_pool,
        })
    }

    /// Builds the request path for the admin create order in pool endpoint
    fn build_admin_create_order_request_path(
        &self,
        non_blocking: bool,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(ADMIN_CREATE_ORDER_IN_POOL_ROUTE, "account_id" => self.get_account_id());
        let query_string =
            serde_urlencoded::to_string(&[(NON_BLOCKING_PARAM, non_blocking.to_string())])
                .map_err(RenegadeClientError::serde)?;

        Ok(format!("{path}?{query_string}"))
    }
}

// -----------------------
// | Admin Order Builder |
// -----------------------

/// The result of building an admin order
#[derive(Debug)]
pub struct BuiltAdminOrder {
    /// The order to be placed
    pub order: ApiOrderCore,
    /// The matching pool to assign the order to
    pub matching_pool: String,
    /// Whether to precompute a cancellation proof for the order
    pub precompute_cancellation_proof: bool,
}

/// Builder for admin order configuration
#[derive(Debug)]
pub struct AdminOrderBuilder {
    /// The owner of the order
    owner: Address,
    /// The ID of the order to create. If not provided, a new UUID will be
    /// generated.
    id: Option<Uuid>,
    /// The input token mint address.
    input_mint: Option<Address>,
    /// The output token mint address.
    output_mint: Option<Address>,
    /// The amount of the input token to trade.
    amount_in: Option<Amount>,
    /// The minimum output token amount that must be received from the order.
    ///
    /// This is used to compute a minimum price (in terms of output token per
    /// input token) below which fills will not execute.
    min_output_amount: Option<Amount>,
    /// The minimum amount that must be filled for the order to execute.
    min_fill_size: Option<Amount>,
    /// The type of order to create.
    order_type: Option<OrderType>,
    /// Whether to allow external matches on the order
    allow_external_matches: Option<bool>,
    /// Whether to precompute a cancellation proof for the order.
    precompute_cancellation_proof: Option<bool>,
    /// The matching pool to assign the order to.
    matching_pool: Option<String>,
}

impl AdminOrderBuilder {
    /// Create a new AdminOrderBuilder with the given owner
    pub(crate) fn new(owner: Address) -> Self {
        Self {
            owner,
            id: None,
            input_mint: None,
            output_mint: None,
            amount_in: None,
            min_output_amount: None,
            min_fill_size: None,
            order_type: None,
            allow_external_matches: None,
            precompute_cancellation_proof: None,
            matching_pool: None,
        }
    }

    /// Set the ID of the order to create
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    /// Set the input mint address
    pub fn with_input_mint(mut self, input_mint: &str) -> Result<Self, RenegadeClientError> {
        let input_mint_address =
            Address::from_str(input_mint).map_err(RenegadeClientError::invalid_order)?;

        self.input_mint = Some(input_mint_address);
        Ok(self)
    }

    /// Set the output mint address
    pub fn with_output_mint(mut self, output_mint: &str) -> Result<Self, RenegadeClientError> {
        let output_mint_address =
            Address::from_str(output_mint).map_err(RenegadeClientError::invalid_order)?;

        self.output_mint = Some(output_mint_address);
        Ok(self)
    }

    /// Set the order input token amount
    pub fn with_input_amount(mut self, amount: Amount) -> Self {
        self.amount_in = Some(amount);
        self
    }

    /// Set the minimum output token amount that must be received from the
    /// order.
    ///
    /// This is used to compute a minimum price (in terms of output token per
    /// input token) below which fills will not execute.
    pub fn with_min_output_amount(mut self, amount: Amount) -> Self {
        self.min_output_amount = Some(amount);
        self
    }

    /// Set the minimum fill size
    pub fn with_min_fill_size(mut self, min_fill: Amount) -> Self {
        self.min_fill_size = Some(min_fill);
        self
    }

    /// Set whether external matches are allowed
    pub fn with_allow_external_matches(mut self, allow: bool) -> Self {
        self.allow_external_matches = Some(allow);
        self
    }

    /// Set the order type, i.e. which level of privacy to prescribe to it.
    pub fn with_order_type(mut self, order_type: OrderType) -> Self {
        self.order_type = Some(order_type);
        self
    }

    /// Set whether to precompute a cancellation proof for the order
    pub fn with_precompute_cancellation_proof(mut self, precompute: bool) -> Self {
        self.precompute_cancellation_proof = Some(precompute);
        self
    }

    /// Set the matching pool to assign the order to
    pub fn with_matching_pool(mut self, matching_pool: String) -> Self {
        self.matching_pool = Some(matching_pool);
        self
    }

    /// Build the admin order, validating all required fields
    pub fn build(self) -> Result<BuiltAdminOrder, RenegadeClientError> {
        let amount_in = unwrap_field!(self, amount_in);

        let min_output_amount: FixedPoint = self.min_output_amount.unwrap_or_default().into();
        let min_price = if min_output_amount == FixedPoint::from(0u64) {
            FixedPoint::from(0u64)
        } else {
            min_output_amount.ceil_div_int(amount_in).into()
        };

        let order = ApiOrderCore {
            id: self.id.unwrap_or_else(Uuid::new_v4),
            intent: ApiIntent {
                in_token: unwrap_field!(self, input_mint),
                out_token: unwrap_field!(self, output_mint),
                owner: self.owner,
                amount_in,
                min_price,
            },
            min_fill_size: self.min_fill_size.unwrap_or(0),
            order_type: unwrap_field!(self, order_type),
            allow_external_matches: self.allow_external_matches.unwrap_or(true),
        };

        let matching_pool = unwrap_field!(self, matching_pool);
        let precompute_cancellation_proof = self.precompute_cancellation_proof.unwrap_or(false);

        Ok(BuiltAdminOrder { order, matching_pool, precompute_cancellation_proof })
    }
}
