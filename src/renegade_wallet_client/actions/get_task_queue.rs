//! Fetches the wallet's current task queue from the relayer

use renegade_api::http::task::{ApiTaskStatus, TaskQueueListResponse, GET_TASK_QUEUE_ROUTE};

use crate::{actions::construct_http_path, client::RenegadeClient, RenegadeClientError};

impl RenegadeClient {
    /// Fetches the list of tasks currently enqueued for the wallet by the
    /// relayer.
    ///
    /// This includes the currently-running task, if there is one.
    pub async fn get_task_queue(&self) -> Result<Vec<ApiTaskStatus>, RenegadeClientError> {
        let wallet_id = self.secrets.wallet_id;
        let path = construct_http_path!(GET_TASK_QUEUE_ROUTE, "wallet_id" => wallet_id);
        let response: TaskQueueListResponse = self.relayer_client.get(&path).await?;
        Ok(response.tasks)
    }
}
