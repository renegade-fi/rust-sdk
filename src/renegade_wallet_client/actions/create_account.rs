//! Create an account with the relayer

use renegade_external_api::EmptyRequestResponse;
use renegade_external_api::http::account::{CREATE_ACCOUNT_ROUTE, CreateAccountRequest};

use crate::{
    RenegadeClientError,
    client::{AccountSecrets, RenegadeClient},
};

impl RenegadeClient {
    /// Create an account with the relayer.
    ///
    /// This method will register the account credentials with the relayer,
    /// but will not yet result in any state being committed onchain in the
    /// darkpool.
    pub async fn create_account(&self) -> Result<(), RenegadeClientError> {
        let AccountSecrets { account_id, master_view_seed, auth_hmac_key, .. } = self.secrets;

        let address = self.get_account_address();
        let schnorr_public_key = self.get_schnorr_public_key();

        let request = CreateAccountRequest {
            account_id,
            address,
            master_view_seed,
            auth_hmac_key,
            schnorr_public_key,
        };

        self.relayer_client.post::<_, EmptyRequestResponse>(CREATE_ACCOUNT_ROUTE, request).await?;

        Ok(())
    }
}
