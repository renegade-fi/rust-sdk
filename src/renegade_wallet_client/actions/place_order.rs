//! Places an order

use std::str::FromStr;

use alloy::primitives::Address;
use renegade_circuit_types::{Amount, fixed_point::FixedPoint};
use renegade_crypto::fields::scalar_to_u256;
use renegade_darkpool_types::{
    balance::{DarkpoolBalance, DarkpoolStateBalance},
    intent::DarkpoolStateIntent,
};
use renegade_external_api::{
    http::order::{CREATE_ORDER_ROUTE, CreateOrderRequest, CreateOrderResponse},
    types::{
        ApiIntent, ApiOrderCore, ApiPublicIntentPermit, OrderAuth, OrderType,
        SignatureWithNonce as ApiSignatureWithNonce,
    },
};
use renegade_solidity_abi::v2::IDarkpoolV2::{PublicIntentPermit, SignatureWithNonce};
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
    /// Create a new order builder with the client's account address as the
    /// owner
    pub fn new_order_builder(&self) -> OrderBuilder {
        OrderBuilder::new(self.get_account_address())
    }

    /// Places an order via the relayer. Waits for the order creation task to
    /// complete before returning the created order.
    ///
    /// Orders will only be committed to onchain state upon their first fill.
    /// As such, this method alone just registers this order as an intent to
    /// trade with the relayer.
    pub async fn place_order(&self, built_order: BuiltOrder) -> Result<(), RenegadeClientError> {
        let request = self.build_create_order_request(built_order).await?;

        let path = self.build_create_order_request_path(false)?;

        self.relayer_client.post::<_, CreateOrderResponse>(&path, request).await?;

        Ok(())
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
        built_order: BuiltOrder,
    ) -> Result<TaskWaiter, RenegadeClientError> {
        let request = self.build_create_order_request(built_order).await?;

        let path = self.build_create_order_request_path(true)?;

        let CreateOrderResponse { task_id, .. } = self.relayer_client.post(&path, request).await?;

        // Create a task waiter for the task
        let task_waiter = self.watch_task(task_id, DEFAULT_TASK_TIMEOUT).await?;
        Ok(task_waiter)
    }
}

// --- Private Helpers --- //
impl RenegadeClient {
    /// Builds the order creation request from the given built order
    async fn build_create_order_request(
        &self,
        built_order: BuiltOrder,
    ) -> Result<CreateOrderRequest, RenegadeClientError> {
        let auth = self.build_order_auth(&built_order.order).await?;

        Ok(CreateOrderRequest {
            order: built_order.order,
            auth,
            precompute_cancellation_proof: built_order.precompute_cancellation_proof,
        })
    }

    /// Builds the order authorization for the given order according to its type
    pub(crate) async fn build_order_auth(
        &self,
        order: &ApiOrderCore,
    ) -> Result<OrderAuth, RenegadeClientError> {
        let intent = order.get_intent();

        // For public orders, we only need to sign over the circuit intent & executor
        // address
        if matches!(order.order_type, OrderType::PublicOrder) {
            let sol_permit =
                PublicIntentPermit { intent: intent.into(), executor: self.get_executor_address() };
            let chain_id = self.get_chain_id();
            let intent_signature = sol_permit
                .sign(chain_id, self.get_account_signer())
                .map_err(RenegadeClientError::signing)?
                .into();
            let permit: ApiPublicIntentPermit = sol_permit.into();

            return Ok(OrderAuth::PublicOrder { permit, intent_signature });
        }

        // For private orders, we need to sample the correct recovery & share stream
        // seeds, then compute a commitment to the intent state object.
        let (mut recovery_seed_csprng, mut share_seed_csprng) = self.get_account_seeds().await?;
        let intent_recovery_stream_seed = recovery_seed_csprng.next().unwrap();
        let intent_share_stream_seed = share_seed_csprng.next().unwrap();

        match order.order_type {
            OrderType::NativelySettledPrivateOrder => {
                // For Ring 1, compute recovery_id first, then compute commitment.
                // The relayer validates using the same ordering.
                let mut state_intent = DarkpoolStateIntent::new(
                    intent,
                    intent_share_stream_seed,
                    intent_recovery_stream_seed,
                );
                state_intent.compute_recovery_id();
                let commitment = state_intent.compute_commitment();

                // Sign the commitment with ECDSA using a nonce
                // SignatureWithNonce::sign internally hashes the payload, so pass raw bytes
                let commitment_u256 = scalar_to_u256(&commitment);
                let chain_id = self.get_chain_id();
                let sig = SignatureWithNonce::sign(
                    &commitment_u256.to_be_bytes::<32>(),
                    chain_id,
                    self.get_account_signer(),
                )
                .map_err(RenegadeClientError::signing)?;
                let intent_signature: ApiSignatureWithNonce = sig.into();
                Ok(OrderAuth::NativelySettledPrivateOrder { intent_signature })
            },
            OrderType::RenegadeSettledPublicFillOrder
            | OrderType::RenegadeSettledPrivateFillOrder => {
                // For Ring 2/3, the circuit expects the signature over the *original*
                // intent commitment (before compute_recovery_id is called).
                let state_intent = DarkpoolStateIntent::new(
                    intent,
                    intent_share_stream_seed,
                    intent_recovery_stream_seed,
                );
                let commitment = state_intent.compute_commitment();

                // Sign the commitment with Schnorr
                let intent_signature = self.schnorr_sign(&commitment)?.into();

                // Renegade-settled orders *may* require the creation of a new output balance,
                // which we authorize optimistically by generating a Schnorr signature over a
                // commitment to the new balance state object.
                let out_token = order.intent.out_token;
                let owner = order.intent.owner;

                let new_output_balance = DarkpoolBalance::new(
                    out_token,
                    owner,
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

                let balance_commitment = state_output_balance.compute_commitment();
                let new_output_balance_signature = self.schnorr_sign(&balance_commitment)?.into();

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
        non_blocking: bool,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(CREATE_ORDER_ROUTE, "account_id" => self.get_account_id());
        let query_string =
            serde_urlencoded::to_string(&[(NON_BLOCKING_PARAM, non_blocking.to_string())])
                .map_err(RenegadeClientError::serde)?;

        Ok(format!("{path}?{query_string}"))
    }
}

// -----------------
// | Order Builder |
// -----------------

/// The result of building an order
#[derive(Debug)]
pub struct BuiltOrder {
    /// The order to be placed
    pub order: ApiOrderCore,
    /// Whether to precompute a cancellation proof for the order
    pub precompute_cancellation_proof: bool,
}

/// Builder for order configuration
#[derive(Debug)]
pub struct OrderBuilder {
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
}

impl OrderBuilder {
    /// Create a new OrderBuilder with the given owner
    pub fn new(owner: Address) -> Self {
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

    /// Build the order, validating all required fields
    pub fn build(self) -> Result<BuiltOrder, RenegadeClientError> {
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

        let precompute_cancellation_proof = self.precompute_cancellation_proof.unwrap_or(false);

        Ok(BuiltOrder { order, precompute_cancellation_proof })
    }
}
