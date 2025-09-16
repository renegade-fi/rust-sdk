//! Gets the balances of the wallet

use alloy::primitives::Address;
use renegade_api::http::wallet::{GetBalanceByMintResponse, GET_BALANCE_BY_MINT_ROUTE};
use renegade_circuit_types::balance::Balance;

use crate::{actions::construct_http_path, client::RenegadeClient, RenegadeClientError};

impl RenegadeClient {
    /// Get the wallet's balance for a given mint
    ///
    /// This method will return the wallet's balance of the mint in the
    /// relayer's current view. Note that this is *not* the back-of-queue
    /// view of the wallet. See [`RenegadeClient::get_wallet`] for more
    /// details.
    pub async fn get_balance_by_mint(&self, mint: Address) -> Result<Balance, RenegadeClientError> {
        let wallet_id = self.secrets.wallet_id;
        let mint_str = mint.to_string();
        let path = construct_http_path!(GET_BALANCE_BY_MINT_ROUTE, "wallet_id" => wallet_id, "mint" => mint_str);
        let response: GetBalanceByMintResponse = self.relayer_client.get(&path).await?;
        Ok(response.balance)
    }
}
