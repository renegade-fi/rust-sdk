//! Create an account with the relayer

use crate::{
    client::{AccountSecrets, RenegadeClient},
    renegade_api_types::{request_response::CreateAccountRequest, CREATE_ACCOUNT_ROUTE},
    RenegadeClientError,
};

impl RenegadeClient {
    /// Create an account with the relayer.
    ///
    /// This method will register the account credentials with the relayer,
    /// but will not yet result in any state being committed onchain in the
    /// darkpool.
    pub async fn create_account(&self) -> Result<(), RenegadeClientError> {
        let AccountSecrets { account_id, master_view_seed, schnorr_key, auth_hmac_key } =
            self.secrets;

        let address = self.get_account_address();

        let request = CreateAccountRequest {
            account_id,
            address,
            master_view_seed,
            schnorr_key,
            auth_hmac_key,
        };

        self.relayer_client.post(CREATE_ACCOUNT_ROUTE, request).await?;

        Ok(())
    }
}
