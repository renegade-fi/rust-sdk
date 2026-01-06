//! Gets the balance of a given mint in the account

use alloy::primitives::Address;

use crate::{
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        balances::ApiBalance, request_response::GetBalanceByMintResponse, GET_BALANCE_BY_MINT_ROUTE,
    },
    RenegadeClientError,
};

impl RenegadeClient {
    /// Get the account's balance for a given mint
    pub async fn get_balance_by_mint(
        &self,
        mint: Address,
    ) -> Result<ApiBalance, RenegadeClientError> {
        let path = construct_http_path!(GET_BALANCE_BY_MINT_ROUTE, "account_id" => self.get_account_id(), "mint" => mint);

        let GetBalanceByMintResponse { balance } = self.relayer_client.get(&path).await?;
        Ok(balance)
    }
}
