//! Sync an account with onchain state

use alloy::primitives::Address;
use renegade_external_api::http::account::{
    SYNC_ACCOUNT_ROUTE, SyncAccountRequest, SyncAccountResponse,
};

use crate::{
    RenegadeClientError,
    actions::{NON_BLOCKING_PARAM, construct_http_path},
    client::RenegadeClient,
    websocket::{DEFAULT_TASK_TIMEOUT, TaskWaiter},
};

// --- Public Actions --- //
impl RenegadeClient {
    /// Sync an account with onchain state. Awaits the completion of the sync
    /// task before returning.
    pub async fn sync_account(&self) -> Result<(), RenegadeClientError> {
        self.sync_account_with_tokens(Vec::new()).await
    }

    /// Sync an account with onchain state, additionally forcing a balance
    /// refresh for the given tokens regardless of whether they appear in
    /// the wallet's active intents. Awaits the completion of the sync
    /// task before returning.
    pub async fn sync_account_with_tokens(
        &self,
        additional_tokens: Vec<Address>,
    ) -> Result<(), RenegadeClientError> {
        let request = self.build_sync_account_request(additional_tokens);

        let path = self.build_sync_account_request_path(false)?;

        self.relayer_client.post::<_, SyncAccountResponse>(&path, request).await?;

        Ok(())
    }

    /// Enqueues a sync task in the relayer. Returns a `TaskWaiter` that can be
    /// used to await task completion.
    pub async fn enqueue_sync_account(&self) -> Result<TaskWaiter, RenegadeClientError> {
        self.enqueue_sync_account_with_tokens(Vec::new()).await
    }

    /// Enqueues a sync task in the relayer, additionally forcing a balance
    /// refresh for the given tokens. Returns a `TaskWaiter` that can be
    /// used to await task completion.
    pub async fn enqueue_sync_account_with_tokens(
        &self,
        additional_tokens: Vec<Address>,
    ) -> Result<TaskWaiter, RenegadeClientError> {
        let request = self.build_sync_account_request(additional_tokens);

        let path = self.build_sync_account_request_path(true)?;

        let SyncAccountResponse { task_id, .. } = self.relayer_client.post(&path, request).await?;

        let task_waiter = self.watch_task(task_id, DEFAULT_TASK_TIMEOUT).await?;

        Ok(task_waiter)
    }
}

// --- Private Helpers --- //
impl RenegadeClient {
    /// Builds the sync account request
    fn build_sync_account_request(
        &self,
        additional_tokens: Vec<Address>,
    ) -> SyncAccountRequest {
        SyncAccountRequest {
            account_id: self.get_account_id(),
            master_view_seed: self.get_master_view_seed(),
            auth_hmac_key: self.get_auth_hmac_key().into(),
            schnorr_public_key: self.get_schnorr_public_key(),
            additional_tokens,
        }
    }

    /// Builds the request path for the sync account endpoint
    fn build_sync_account_request_path(
        &self,
        non_blocking: bool,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(SYNC_ACCOUNT_ROUTE, "account_id" => self.get_account_id());
        let query_string =
            serde_urlencoded::to_string(&[(NON_BLOCKING_PARAM, non_blocking.to_string())])
                .map_err(RenegadeClientError::serde)?;

        Ok(format!("{path}?{query_string}"))
    }
}
