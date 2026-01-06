//! Gets all of the balances in the account

use crate::{
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        balances::ApiBalance, request_response::GetBalancesResponse, GET_BALANCES_ROUTE,
    },
    RenegadeClientError,
};

impl RenegadeClient {
    /// Fetches all balances in the account.
    pub async fn get_balances(&self) -> Result<Vec<ApiBalance>, RenegadeClientError> {
        let path = construct_http_path!(GET_BALANCES_ROUTE, "account_id" => self.get_account_id());

        let GetBalancesResponse { balances } = self.relayer_client.get(&path).await?;

        Ok(balances)
    }
}
