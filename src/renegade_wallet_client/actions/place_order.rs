//! Places an order in the wallet

use num_bigint::BigUint;
use renegade_api::{
    http::wallet::{CreateOrderRequest, CreateOrderResponse, WALLET_ORDERS_ROUTE},
    types::{ApiOrder, ApiOrderType},
};
use renegade_circuit_types::{fixed_point::FixedPoint, max_price, order::OrderSide};
use renegade_common::types::wallet::Order;
use renegade_utils::hex::biguint_from_hex_string;
use uuid::Uuid;

use crate::{
    actions::{construct_http_path, prepare_wallet_update},
    client::RenegadeClient,
    websocket::TaskWaiter,
    RenegadeClientError,
};

impl RenegadeClient {
    /// Place an order in the wallet
    pub async fn place_order(&self, order: ApiOrder) -> Result<TaskWaiter, RenegadeClientError> {
        // Add the order to the wallet
        let mut wallet = self.get_internal_wallet().await?;
        let internal_order =
            Order::try_from(order.clone()).map_err(RenegadeClientError::conversion)?;
        wallet.add_order(order.id, internal_order).map_err(RenegadeClientError::wallet)?;

        // Update the wallet auth
        let wallet_id = self.secrets.wallet_id;
        let update_auth = prepare_wallet_update(&mut wallet)?;
        let request = CreateOrderRequest { update_auth, order };

        let route = construct_http_path!(WALLET_ORDERS_ROUTE, "wallet_id" => wallet_id);
        let response: CreateOrderResponse = self.post_relayer(&route, request).await?;

        // Create a task waiter for the task
        let task_id = response.task_id;
        Ok(self.get_task_waiter(task_id))
    }
}

// -----------------
// | Order Builder |
// -----------------

/// Builder for creating ApiOrder instances
#[derive(Debug, Default)]
pub struct OrderBuilder {
    /// The base token mint address.
    base_mint: Option<BigUint>,
    /// The quote token mint address.
    quote_mint: Option<BigUint>,
    /// The order side (Buy or Sell).
    side: Option<OrderSide>,
    /// The amount of the base token to trade.
    amount: Option<u128>,
    /// The worst case price the trader is willing to accept.
    worst_case_price: Option<FixedPoint>,
    /// The minimum amount that must be filled for the order to execute.
    min_fill_size: Option<u128>,
    /// Whether this order can be matched with external counterparties.
    allow_external_matches: Option<bool>,
}

impl OrderBuilder {
    /// Create a new OrderBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base mint address
    pub fn with_base_mint(mut self, base_mint: &str) -> Result<Self, RenegadeClientError> {
        let hex_mint =
            biguint_from_hex_string(base_mint).map_err(RenegadeClientError::invalid_order)?;
        self.base_mint = Some(hex_mint);
        Ok(self)
    }

    /// Set the quote mint address
    pub fn with_quote_mint(mut self, quote_mint: &str) -> Result<Self, RenegadeClientError> {
        let hex_mint =
            biguint_from_hex_string(quote_mint).map_err(RenegadeClientError::invalid_order)?;
        self.quote_mint = Some(hex_mint);
        Ok(self)
    }

    /// Set the order side (Buy or Sell)
    pub fn with_side(mut self, side: OrderSide) -> Self {
        self.side = Some(side);
        self
    }

    /// Set the order amount
    pub fn with_amount(mut self, amount: u128) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the worst case price
    pub fn with_worst_case_price(mut self, price: f64) -> Self {
        self.worst_case_price = Some(FixedPoint::from_f64_round_down(price));
        self
    }

    /// Set the minimum fill size
    pub fn with_min_fill_size(mut self, min_fill: u128) -> Self {
        self.min_fill_size = Some(min_fill);
        self
    }

    /// Set whether external matches are allowed
    pub fn with_allow_external_matches(mut self, allow: bool) -> Self {
        self.allow_external_matches = Some(allow);
        self
    }

    /// Build the ApiOrder
    pub fn build(self) -> Result<ApiOrder, RenegadeClientError> {
        let id = Uuid::new_v4();
        let worst_case_price = match (self.worst_case_price, self.side) {
            (Some(price), _) => price,
            (None, Some(OrderSide::Buy)) => max_price(),
            (None, Some(OrderSide::Sell)) => FixedPoint::zero(),
            _ => return Err(RenegadeClientError::invalid_order("side is required")),
        };

        macro_rules! unwrap_field {
            ($field:ident) => {
                self.$field.ok_or_else(|| {
                    RenegadeClientError::invalid_order(format!(
                        "{} is required for order",
                        stringify!($field)
                    ))
                })?
            };
        }

        Ok(ApiOrder {
            id,
            base_mint: unwrap_field!(base_mint),
            quote_mint: unwrap_field!(quote_mint),
            side: unwrap_field!(side),
            amount: unwrap_field!(amount),
            worst_case_price,
            min_fill_size: self.min_fill_size.unwrap_or(0),
            type_: ApiOrderType::Midpoint,
            allow_external_matches: self.allow_external_matches.unwrap_or(true),
        })
    }
}
