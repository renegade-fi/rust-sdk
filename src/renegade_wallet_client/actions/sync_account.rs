//! Sync an account with onchain state

use crate::{
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        request_response::{SyncAccountQueryParameters, SyncAccountRequest, SyncAccountResponse},
        SYNC_ACCOUNT_ROUTE,
    },
    websocket::TaskWaiter,
    RenegadeClientError,
};

// --- Public Actions --- //
impl RenegadeClient {
    /// Sync an account with onchain state. Awaits the completion of the sync
    /// task before returning.
    pub async fn sync_account(&self) -> Result<(), RenegadeClientError> {
        let request = SyncAccountRequest {
            account_id: self.get_account_id(),
            master_view_seed: self.get_master_view_seed(),
            auth_hmac_key: self.get_auth_hmac_key(),
        };

        let query_params = SyncAccountQueryParameters { non_blocking: Some(false) };
        let path = self.build_sync_account_request_path(&query_params)?;

        self.relayer_client.post::<_, SyncAccountResponse>(&path, request).await?;

        Ok(())
    }

    /// Enqueues a sync task in the relayer. Returns a `TaskWaiter` that can be
    /// used to await task completion.
    pub async fn enqueue_sync_account(&self) -> Result<TaskWaiter, RenegadeClientError> {
        let request = SyncAccountRequest {
            account_id: self.get_account_id(),
            master_view_seed: self.get_master_view_seed(),
            auth_hmac_key: self.get_auth_hmac_key(),
        };

        let query_params = SyncAccountQueryParameters { non_blocking: Some(true) };
        let path = self.build_sync_account_request_path(&query_params)?;

        let SyncAccountResponse { task_id, .. } = self.relayer_client.post(&path, request).await?;

        Ok(self.get_default_task_waiter(task_id))
    }
}

// --- Private Helpers --- //
impl RenegadeClient {
    /// Builds the request to sync an account
    fn build_sync_account_request_path(
        &self,
        query_params: &SyncAccountQueryParameters,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(SYNC_ACCOUNT_ROUTE, "account_id" => self.get_account_id());
        let query_string =
            serde_urlencoded::to_string(query_params).map_err(RenegadeClientError::serde)?;

        Ok(format!("{}?{}", path, query_string))
    }
}
