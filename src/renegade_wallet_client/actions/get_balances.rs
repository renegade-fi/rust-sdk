//! Gets all of the balances in the account

use renegade_external_api::{
    http::balance::{GET_BALANCES_ROUTE, GetBalancesResponse},
    types::ApiBalance,
};

use crate::{RenegadeClientError, actions::construct_http_path, client::RenegadeClient};

impl RenegadeClient {
    /// Fetches all balances in the account.
    pub async fn get_balances(&self) -> Result<Vec<ApiBalance>, RenegadeClientError> {
        let path = construct_http_path!(GET_BALANCES_ROUTE, "account_id" => self.get_account_id());

        let GetBalancesResponse { balances } = self.relayer_client.get(&path).await?;

        Ok(balances)
    }
}
