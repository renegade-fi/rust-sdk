//! Updates an order

use renegade_circuit_types::Amount;
use uuid::Uuid;

use crate::{
    client::RenegadeClient,
    renegade_api_types::{
        orders::{ApiOrder, ApiOrderCore},
        request_response::{UpdateOrderRequest, UpdateOrderResponse},
        UPDATE_ORDER_ROUTE,
    },
    RenegadeClientError,
};

impl RenegadeClient {
    /// Updates an order.
    ///
    /// Currently, the only parameters which can updated are the order's
    /// `min_fill_size`, and whether to `allow_external_matches`.
    pub async fn update_order(
        &self,
        order_update_config: OrderUpdateConfig,
    ) -> Result<ApiOrder, RenegadeClientError> {
        let request = self.build_request(order_update_config).await?;
        let response: UpdateOrderResponse =
            self.relayer_client.post(&UPDATE_ORDER_ROUTE, request).await?;

        Ok(response.order)
    }

    /// Builds the order update request
    async fn build_request(
        &self,
        order_update_config: OrderUpdateConfig,
    ) -> Result<UpdateOrderRequest, RenegadeClientError> {
        let mut order = match order_update_config.initial_order {
            Some(initial_order) => initial_order,
            None => self.get_order(order_update_config.order_id).await?.into(),
        };

        if let Some(min_fill_size) = order_update_config.min_fill_size {
            order.min_fill_size = min_fill_size;
        }

        if let Some(allow_external_matches) = order_update_config.allow_external_matches {
            order.allow_external_matches = allow_external_matches;
        }

        Ok(UpdateOrderRequest { order })
    }
}

// -----------------------
// | Order Update Config |
// -----------------------

/// Container for order update configuration options
#[derive(Debug, Default)]
pub struct OrderUpdateConfig {
    /// The ID of the order to update.
    order_id: Uuid,
    /// The initial order to update. If not provided, the order will be fetched
    /// from the relayer.
    initial_order: Option<ApiOrderCore>,
    /// The updated minimum fill size for the order.
    min_fill_size: Option<Amount>,
    /// Whether to allow external matches on the order.
    allow_external_matches: Option<bool>,
}

impl OrderUpdateConfig {
    /// Create a new order update config
    pub fn new(order_id: Uuid) -> Self {
        Self { order_id, ..Default::default() }
    }

    /// Set the initial order to update
    pub fn with_initial_order(
        mut self,
        initial_order: ApiOrderCore,
    ) -> Result<Self, RenegadeClientError> {
        if initial_order.id != self.order_id {
            return Err(RenegadeClientError::invalid_order_update(format!(
                "Initial order ID does not match the order ID to update: {} != {}",
                initial_order.id, self.order_id
            )));
        }

        self.initial_order = Some(initial_order);
        Ok(self)
    }

    /// Set the updated minimum fill size for the order
    pub fn with_min_fill_size(mut self, min_fill_size: Amount) -> Self {
        self.min_fill_size = Some(min_fill_size);
        self
    }

    /// Set whether to allow external matches on the order
    pub fn with_allow_external_matches(mut self, allow_external_matches: bool) -> Self {
        self.allow_external_matches = Some(allow_external_matches);
        self
    }
}
