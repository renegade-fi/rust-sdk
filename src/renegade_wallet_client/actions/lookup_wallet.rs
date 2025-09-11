//! Looks up a wallet onchain

use renegade_api::{
    http::wallet::{FindWalletRequest, FindWalletResponse, FIND_WALLET_ROUTE},
    types::ApiKeychain,
};

use crate::{client::RenegadeClient, websocket::TaskWaiter, RenegadeClientError};

impl RenegadeClient {
    /// Looks up a wallet on-chain, indexing its most recent state in the
    /// relayer.
    ///
    /// Under the hood, the relayer reconstructs the wallet from the public
    /// shares included in the calldata of the last wallet update, using the
    /// provided wallet secrets.
    pub async fn lookup_wallet(&self) -> Result<TaskWaiter, RenegadeClientError> {
        let wallet_id = self.secrets.wallet_id;
        let blinder_seed = self.secrets.blinder_seed.into();
        let secret_share_seed = self.secrets.share_seed.into();
        let api_keychain: ApiKeychain = self.secrets.keychain.clone().into();
        let private_keychain = api_keychain.private_keys;

        let request =
            FindWalletRequest { wallet_id, blinder_seed, secret_share_seed, private_keychain };

        let response: FindWalletResponse = self.post_relayer(FIND_WALLET_ROUTE, request).await?;

        let task_id = response.task_id;
        Ok(self.get_task_waiter(task_id))
    }
}
