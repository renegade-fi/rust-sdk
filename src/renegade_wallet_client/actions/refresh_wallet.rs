//! Refreshes a wallet's state in the relayer

use renegade_api::{
    http::wallet::{RefreshWalletResponse, REFRESH_WALLET_ROUTE},
    EmptyRequestResponse,
};

use crate::{
    actions::construct_http_path, client::RenegadeClient, websocket::TaskWaiter,
    RenegadeClientError,
};

impl RenegadeClient {
    /// Refreshes a wallet's state in the relayer, ensuring that it's up-to-date
    /// with onchain state & clearing its task queue.
    pub async fn refresh_wallet(&self) -> Result<TaskWaiter, RenegadeClientError> {
        let wallet_id = self.secrets.wallet_id;
        let route = construct_http_path!(REFRESH_WALLET_ROUTE, "wallet_id" => wallet_id);
        let response: RefreshWalletResponse =
            self.relayer_client.post(&route, EmptyRequestResponse {}).await?;

        // Create a task waiter for the task
        let task_id = response.task_id;
        Ok(self.get_task_waiter(task_id))
    }
}
