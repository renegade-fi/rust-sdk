//! Create the wallet in the darkpool

use renegade_api::{
    http::wallet::{CreateWalletRequest, CreateWalletResponse, CREATE_WALLET_ROUTE},
    types::ApiWallet,
};

use crate::{client::RenegadeClient, websocket::TaskWaiter, RenegadeClientError};

impl RenegadeClient {
    /// Create the wallet in the darkpool
    ///
    /// This method will ask a relayer to allocate the wallet's state in the
    /// darkpool as an empty wallet derived from the user's key.
    pub async fn create_wallet(&self) -> Result<TaskWaiter, RenegadeClientError> {
        let wallet = self.secrets.generate_empty_wallet();
        let api_wallet = ApiWallet::from(wallet);
        let blinder_seed = self.secrets.blinder_seed.to_biguint();
        let request = CreateWalletRequest { wallet: api_wallet, blinder_seed };

        let response: CreateWalletResponse =
            self.post_relayer(CREATE_WALLET_ROUTE, request).await?;

        // Extract task_id from response and create task waiter
        let task_id = response.task_id;
        self.websocket_client.watch_task(task_id).await
    }
}
