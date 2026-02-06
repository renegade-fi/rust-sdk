//! Cancels an order in the wallet

use alloy::{
    primitives::{U256, keccak256},
    signers::SignerSync,
};
use renegade_darkpool_types::intent::DarkpoolStateIntent;
use renegade_external_api::{
    http::order::{CANCEL_ORDER_ROUTE, CancelOrderRequest, CancelOrderResponse},
    types::SignatureWithNonce,
};
use uuid::Uuid;

use crate::{
    RenegadeClientError,
    actions::{NON_BLOCKING_PARAM, construct_http_path},
    client::RenegadeClient,
    websocket::{DEFAULT_TASK_TIMEOUT, TaskWaiter},
};

// --- Public Actions --- //
impl RenegadeClient {
    /// Cancels the order with the given ID. Waits for the order cancellation
    /// task to complete before returning.
    pub async fn cancel_order(&self, order_id: Uuid) -> Result<(), RenegadeClientError> {
        let request = self.build_cancel_order_request(order_id).await?;

        let path = self.build_cancel_order_request_path(order_id, false)?;

        self.relayer_client.post::<_, CancelOrderResponse>(&path, request).await?;

        Ok(())
    }

    /// Enqueues an order cancellation task in the relayer. Returns a
    /// `TaskWaiter` that can be used to await task completion.
    pub async fn enqueue_order_cancellation(
        &self,
        order_id: Uuid,
    ) -> Result<TaskWaiter, RenegadeClientError> {
        let request = self.build_cancel_order_request(order_id).await?;

        let path = self.build_cancel_order_request_path(order_id, true)?;

        let CancelOrderResponse { task_id, .. } = self.relayer_client.post(&path, request).await?;

        // Create a task waiter for the task
        let task_waiter = self.watch_task(task_id, DEFAULT_TASK_TIMEOUT).await?;

        Ok(task_waiter)
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

        // Generate a nonce for the cancellation signature
        let nonce = rand::random::<u64>();

        let signature = self
            .get_account_signer()
            .sign_hash_sync(&nullifier_hash)
            .map_err(RenegadeClientError::signing)?;

        let cancel_signature = SignatureWithNonce {
            nonce: U256::from(nonce),
            signature: signature.as_bytes().to_vec(),
        };

        Ok(CancelOrderRequest { cancel_signature })
    }

    /// Builds the request path for the cancel order endpoint
    fn build_cancel_order_request_path(
        &self,
        order_id: Uuid,
        non_blocking: bool,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(CANCEL_ORDER_ROUTE, "account_id" => self.get_account_id(), "order_id" => order_id);
        let query_string =
            serde_urlencoded::to_string(&[(NON_BLOCKING_PARAM, non_blocking.to_string())])
                .map_err(RenegadeClientError::serde)?;

        Ok(format!("{path}?{query_string}"))
    }
}
