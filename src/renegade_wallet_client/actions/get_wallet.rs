//! Gets the latest wallet state from the relayer

use renegade_api::{
    http::wallet::{GetWalletResponse, BACK_OF_QUEUE_WALLET_ROUTE},
    types::ApiWallet,
};
use renegade_common::types::wallet::Wallet;

use crate::{actions::construct_http_path, client::RenegadeClient, RenegadeClientError};

/// Get the Renegade wallet state from the relayer
///
/// This action pulls the "back-of-queue" wallet for the client. So if there are
/// any pending wallet tasks, this action will return the state assuming that
/// the tasks complete
impl RenegadeClient {
    /// Get the latest wallet state from the relayer
    pub async fn get_wallet(&self) -> Result<ApiWallet, RenegadeClientError> {
        let id = self.secrets.wallet_id;
        let path = construct_http_path!(BACK_OF_QUEUE_WALLET_ROUTE, "wallet_id" => id);
        let response: GetWalletResponse = self.get_relayer(&path).await?;
        Ok(response.wallet)
    }

    /// An internal helper to get the back of queue wallet as a
    /// `renegade_common::Wallet`
    pub(crate) async fn get_internal_wallet(&self) -> Result<Wallet, RenegadeClientError> {
        let wallet = self.get_wallet().await?;
        wallet.try_into().map_err(RenegadeClientError::conversion)
    }
}
