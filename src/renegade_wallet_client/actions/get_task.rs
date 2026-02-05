//! Looks up a task by its ID in the relayer

use renegade_external_api::{
    http::task::{GET_TASK_BY_ID_ROUTE, GetTaskByIdResponse},
    types::ApiTask,
};
use uuid::Uuid;

use crate::{RenegadeClientError, actions::construct_http_path, client::RenegadeClient};

// --- Public Actions --- //
impl RenegadeClient {
    /// Look up a task by its ID
    pub async fn get_task(&self, task_id: Uuid) -> Result<ApiTask, RenegadeClientError> {
        let path = construct_http_path!(GET_TASK_BY_ID_ROUTE, "account_id" => self.get_account_id(), "task_id" => task_id);

        let GetTaskByIdResponse { task } = self.relayer_client.get(&path).await?;
        Ok(task)
    }
}
