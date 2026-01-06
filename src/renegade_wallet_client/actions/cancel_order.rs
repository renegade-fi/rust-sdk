//! Cancels an order in the wallet

use alloy::{primitives::keccak256, signers::SignerSync};
use renegade_circuit_types::intent::DarkpoolStateIntent;
use uuid::Uuid;

use crate::{
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        orders::ApiOrder,
        request_response::{CancelOrderQueryParameters, CancelOrderRequest, CancelOrderResponse},
        CANCEL_ORDER_ROUTE,
    },
    websocket::{TaskWaiter, DEFAULT_TASK_TIMEOUT},
    RenegadeClientError,
};

// --- Public Actions --- //
impl RenegadeClient {
    /// Cancels the order with the given ID. Waits for the order cancellation
    /// task to complete before returning the cancelled order.
    pub async fn cancel_order(&self, order_id: Uuid) -> Result<ApiOrder, RenegadeClientError> {
        let request = self.build_cancel_order_request(order_id).await?;

        let query_params = CancelOrderQueryParameters { non_blocking: Some(false) };
        let path = self.build_cancel_order_request_path(order_id, &query_params)?;

        let CancelOrderResponse { order, .. } = self.relayer_client.post(&path, request).await?;

        Ok(order)
    }

    /// Enqueues an order cancellation task in the relayer. Returns the
    /// cancelled order, and a `TaskWaiter` that can be used to await task
    /// completion.
    pub async fn enqueue_order_cancellation(
        &self,
        order_id: Uuid,
    ) -> Result<(ApiOrder, TaskWaiter), RenegadeClientError> {
        let request = self.build_cancel_order_request(order_id).await?;

        let query_params = CancelOrderQueryParameters { non_blocking: Some(true) };
        let path = self.build_cancel_order_request_path(order_id, &query_params)?;

        let CancelOrderResponse { task_id, order, .. } =
            self.relayer_client.post(&path, request).await?;

        // Create a task waiter for the task
        let task_waiter = self.watch_task(task_id, DEFAULT_TASK_TIMEOUT).await?;

        Ok((order, task_waiter))
    }
}

// --- Private Helpers --- //
impl RenegadeClient {
    /// Builds the order cancellation request from the given order ID
    async fn build_cancel_order_request(
        &self,
        order_id: Uuid,
    ) -> Result<CancelOrderRequest, RenegadeClientError> {
        let order = self.get_order(order_id).await?;
        let intent: DarkpoolStateIntent = order.into();

        let nullifier = intent.compute_nullifier();
        let nullifier_hash = keccak256(nullifier.to_bytes_be());
        let signature = self
            .get_account_signer()
            .sign_hash_sync(&nullifier_hash)
            .map_err(RenegadeClientError::signing)?
            .into();

        Ok(CancelOrderRequest { signature })
    }

    /// Builds the request path for the cancel order endpoint
    fn build_cancel_order_request_path(
        &self,
        order_id: Uuid,
        query_params: &CancelOrderQueryParameters,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(CANCEL_ORDER_ROUTE, "account_id" => self.get_account_id(), "order_id" => order_id);
        let query_string =
            serde_urlencoded::to_string(query_params).map_err(RenegadeClientError::serde)?;

        Ok(format!("{}?{}", path, query_string))
    }
}
