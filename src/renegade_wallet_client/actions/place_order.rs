//! Places an order

use std::str::FromStr;

use alloy::primitives::Address;
use renegade_circuit_types::{
    balance::{Balance, DarkpoolStateBalance},
    fixed_point::FixedPoint,
    intent::DarkpoolStateIntent,
    Amount,
};
use renegade_solidity_abi::v2::IDarkpoolV2::{self, PublicIntentPermit};
use uuid::Uuid;

use crate::{
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        orders::{ApiOrder, ApiOrderCore, OrderAuth, OrderType},
        request_response::{CreateOrderQueryParameters, CreateOrderRequest, CreateOrderResponse},
        CREATE_ORDER_ROUTE,
    },
    websocket::TaskWaiter,
    RenegadeClientError,
};

// --- Public Actions --- //
impl RenegadeClient {
    /// Places an order via the relayer. Waits for the order creation task to
    /// complete before returning the created order.
    ///
    /// Orders will only be committed to onchain state upon their first fill.
    /// As such, this method alone just registers this order as an intent to
    /// trade with the relayer.
    pub async fn place_order(
        &self,
        order_config: OrderConfig,
    ) -> Result<ApiOrder, RenegadeClientError> {
        let request = self.build_create_order_request(order_config).await?;

        let query_params = CreateOrderQueryParameters { non_blocking: Some(false) };
        let path = self.build_create_order_request_path(&query_params)?;

        let CreateOrderResponse { order, .. } = self.relayer_client.post(&path, request).await?;

        Ok(order)
    }

    /// Enqueues an order placement task in the relayer. Returns the expected
    /// order to be created, and a `TaskWaiter` that can be used to await task
    /// completion.
    ///
    /// Orders will only be committed to onchain state upon their first fill.
    /// As such, this method alone just registers this order as an intent to
    /// trade with the relayer.
    pub async fn enqueue_order_placement(
        &self,
        order_config: OrderConfig,
    ) -> Result<(ApiOrder, TaskWaiter), RenegadeClientError> {
        let request = self.build_create_order_request(order_config).await?;

        let query_params = CreateOrderQueryParameters { non_blocking: Some(true) };
        let path = self.build_create_order_request_path(&query_params)?;

        let CreateOrderResponse { task_id, order, .. } =
            self.relayer_client.post(&path, request).await?;

        // Create a task waiter for the task
        Ok((order, self.get_default_task_waiter(task_id)))
    }
}

// --- Private Helpers --- //
impl RenegadeClient {
    /// Builds the order creation request from the given order configuration
    async fn build_create_order_request(
        &self,
        order_config: OrderConfig,
    ) -> Result<CreateOrderRequest, RenegadeClientError> {
        let precompute_cancellation_proof =
            order_config.precompute_cancellation_proof.unwrap_or(false);

        let order = self.build_order(order_config)?;
        let auth = self.build_order_auth(&order).await?;

        Ok(CreateOrderRequest { order, auth, precompute_cancellation_proof })
    }

    /// Builds an ApiOrderCore from an OrderConfig, injecting the client's
    /// address as the owner
    fn build_order(&self, config: OrderConfig) -> Result<ApiOrderCore, RenegadeClientError> {
        macro_rules! unwrap_field {
            ($field:ident) => {
                config.$field.ok_or_else(|| {
                    RenegadeClientError::invalid_order(format!(
                        "{} is required for order",
                        stringify!($field)
                    ))
                })?
            };
        }

        let amount_in = unwrap_field!(amount_in);

        let min_output_amount: FixedPoint = config.min_output_amount.unwrap_or_default().into();
        let min_price = min_output_amount.ceil_div_int(amount_in).into();

        Ok(ApiOrderCore {
            id: Uuid::new_v4(),
            in_token: unwrap_field!(input_mint),
            out_token: unwrap_field!(output_mint),
            owner: self.get_account_address(),
            amount_in: unwrap_field!(amount_in),
            min_price,
            min_fill_size: config.min_fill_size.unwrap_or(0),
            order_type: unwrap_field!(order_type),
            allow_external_matches: config.allow_external_matches.unwrap_or(true),
        })
    }

    /// Builds the order authorization for the given order according to its type
    async fn build_order_auth(
        &self,
        order: &ApiOrderCore,
    ) -> Result<OrderAuth, RenegadeClientError> {
        let intent: IDarkpoolV2::Intent = order.into();

        // For public orders, we only need to sign over the circuit intent & executor
        // address
        if matches!(order.order_type, OrderType::PublicOrder) {
            let permit = PublicIntentPermit { intent, executor: self.get_executor_address() };
            let intent_signature = permit
                .sign(self.get_account_signer())
                .map_err(RenegadeClientError::signing)?
                .into();

            return Ok(OrderAuth::PublicOrder { intent_signature });
        }

        // For private orders, we need to sample the correct recovery & share stream
        // seeds, then generate a Schnorr signature over the commitment to the
        // intent state object.
        let (mut recovery_seed_csprng, mut share_seed_csprng) = self.get_account_seeds().await?;
        let intent_recovery_stream_seed = recovery_seed_csprng.next().unwrap();
        let intent_share_stream_seed = share_seed_csprng.next().unwrap();

        let state_intent = DarkpoolStateIntent::new(
            intent.into(),
            intent_share_stream_seed,
            intent_recovery_stream_seed,
        );

        let commitment = state_intent.compute_commitment();
        let intent_signature = self.schnorr_sign(&commitment)?.into();

        match order.order_type {
            OrderType::NativelySettledPrivateOrder => {
                Ok(OrderAuth::NativelySettledPrivateOrder { intent_signature })
            },
            OrderType::RenegadeSettledPublicFillOrder
            | OrderType::RenegadeSettledPrivateFillOrder => {
                // Renegade-settled orders *may* require the creation of a new output balance,
                // which we authorize optimistically by generating a Schnorr signature over a
                // commitment to the new balance state object.
                let new_output_balance = Balance::new(
                    order.out_token,
                    order.owner,
                    self.get_relayer_fee_recipient(),
                    self.get_schnorr_public_key(),
                );

                let balance_recovery_stream_seed = recovery_seed_csprng.next().unwrap();
                let balance_share_stream_seed = share_seed_csprng.next().unwrap();

                let state_output_balance = DarkpoolStateBalance::new(
                    new_output_balance,
                    balance_share_stream_seed,
                    balance_recovery_stream_seed,
                );

                let commitment = state_output_balance.compute_commitment();
                let new_output_balance_signature = self.schnorr_sign(&commitment)?.into();

                Ok(OrderAuth::RenegadeSettledOrder {
                    intent_signature,
                    new_output_balance_signature,
                })
            },
            OrderType::PublicOrder => unreachable!(),
        }
    }

    /// Builds the request path for the create order endpoint
    fn build_create_order_request_path(
        &self,
        query_params: &CreateOrderQueryParameters,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(CREATE_ORDER_ROUTE, "account_id" => self.get_account_id());
        let query_string =
            serde_urlencoded::to_string(&query_params).map_err(RenegadeClientError::serde)?;

        Ok(format!("{}?{}", path, query_string))
    }
}

// ----------------
// | Order Config |
// ----------------

/// Container for order configuration options
#[derive(Debug, Default)]
pub struct OrderConfig {
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
}

impl OrderConfig {
    /// Create a new OrderConfig
    pub fn new() -> Self {
        Self::default()
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

    pub fn with_precompute_cancellation_proof(mut self, precompute: bool) -> Self {
        self.precompute_cancellation_proof = Some(precompute);
        self
    }
}
