//! Looks up an account by its ID in the relayer

use crate::{
    RenegadeClientError,
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        GET_ACCOUNT_BY_ID_ROUTE, account::ApiAccount, request_response::GetAccountResponse,
    },
};

impl RenegadeClient {
    /// Look up an account by its ID
    ///
    /// Returns the account's orders and balances
    pub async fn get_account(&self) -> Result<ApiAccount, RenegadeClientError> {
        let path =
            construct_http_path!(GET_ACCOUNT_BY_ID_ROUTE, "account_id" => self.get_account_id());
        let GetAccountResponse { account } = self.relayer_client.get(&path).await?;
        Ok(account)
    }
}
