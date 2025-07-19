//! Cancels an order in the wallet

use renegade_api::http::wallet::{CancelOrderRequest, CancelOrderResponse, CANCEL_ORDER_ROUTE};
use uuid::Uuid;

use crate::{
    actions::{construct_http_path, prepare_wallet_update},
    client::RenegadeClient,
    websocket::TaskWaiter,
    RenegadeClientError,
};

impl RenegadeClient {
    /// Cancels an order in the wallet
    pub async fn cancel_order(&self, order_id: Uuid) -> Result<TaskWaiter, RenegadeClientError> {
        let mut wallet = self.get_internal_wallet().await?;
        if !wallet.contains_order(&order_id) {
            return Err(RenegadeClientError::wallet(format!(
                "Order {order_id} not found in wallet"
            )));
        }

        wallet.remove_order(&order_id).unwrap();
        let update_auth = prepare_wallet_update(&mut wallet)?;

        // Send the request
        let route = construct_http_path!(CANCEL_ORDER_ROUTE, "wallet_id" => self.secrets.wallet_id, "order_id" => order_id);
        let request = CancelOrderRequest { update_auth };
        let response: CancelOrderResponse = self.post_relayer(&route, request).await?;

        // Create a task waiter for the task
        let task_id = response.task_id;
        Ok(self.get_task_waiter(task_id))
    }
}
