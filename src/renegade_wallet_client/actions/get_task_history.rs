//! Gets the wallet's task history from the historical state engine

use renegade_api::{
    http::task_history::{GetTaskHistoryResponse, TASK_HISTORY_ROUTE},
    types::ApiHistoricalTask,
};
use renegade_common::types::wallet::WalletIdentifier;

use crate::{
    actions::construct_http_path, client::RenegadeClient, http::RelayerHttpClient,
    RenegadeClientError,
};

impl RenegadeClient {
    /// Get the wallet's task history from the historical state engine
    pub async fn get_task_history(&self) -> Result<Vec<ApiHistoricalTask>, RenegadeClientError> {
        get_task_history(&self.historical_state_client, self.secrets.wallet_id).await
    }
}

/// Request task history for the given wallet on the given HTTP client
// TODO: Implement response pagination
pub async fn get_task_history(
    client: &RelayerHttpClient,
    wallet_id: WalletIdentifier,
) -> Result<Vec<ApiHistoricalTask>, RenegadeClientError> {
    let path = construct_http_path!(TASK_HISTORY_ROUTE, "wallet_id" => wallet_id);
    let response: GetTaskHistoryResponse = client.get(&path).await?;
    Ok(response.tasks)
}
