//! Gets the balance of a given mint in the account

use alloy::primitives::Address;
use renegade_external_api::{
    http::balance::{GET_BALANCE_BY_MINT_ROUTE, GetBalanceByMintResponse},
    types::ApiBalance,
};

use crate::{RenegadeClientError, actions::construct_http_path, client::RenegadeClient};

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
