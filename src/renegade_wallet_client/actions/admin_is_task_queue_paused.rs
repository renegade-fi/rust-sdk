//! Check if an account's task queue is paused (admin)

use renegade_external_api::http::admin::ADMIN_GET_TASK_QUEUE_PAUSED_ROUTE;
use renegade_external_api::types::TaskQueuePausedResponse;
use uuid::Uuid;

use crate::{RenegadeClientError, actions::construct_http_path, client::RenegadeClient};

impl RenegadeClient {
    /// Check if the given account's task queue is paused
    pub async fn admin_is_task_queue_paused(
        &self,
        account_id: Uuid,
    ) -> Result<bool, RenegadeClientError> {
        let admin_relayer_client = self.get_admin_client()?;
        let path = construct_http_path!(
            ADMIN_GET_TASK_QUEUE_PAUSED_ROUTE,
            "account_id" => account_id
        );

        let TaskQueuePausedResponse { paused } = admin_relayer_client.get(&path).await?;
        Ok(paused)
    }
}
