//! Admin action to set or clear an account's default matching pool.

use renegade_external_api::{
    EmptyRequestResponse,
    http::admin::{
        ADMIN_SET_ACCOUNT_DEFAULT_POOL_ROUTE, SetAccountDefaultMatchingPoolRequest,
    },
};
use uuid::Uuid;

use crate::{RenegadeClientError, actions::construct_http_path, client::RenegadeClient};

impl RenegadeClient {
    /// Set the default matching pool for an account, or pass `None` to clear
    /// the binding so future orders fall back to the global pool.
    ///
    /// This is an admin action that requires the client to be configured with
    /// an admin HMAC key.
    pub async fn admin_set_account_default_pool(
        &self,
        account_id: Uuid,
        matching_pool: Option<String>,
    ) -> Result<(), RenegadeClientError> {
        let admin_client = self.get_admin_client()?;

        let path = construct_http_path!(
            ADMIN_SET_ACCOUNT_DEFAULT_POOL_ROUTE,
            "account_id" => account_id
        );

        let body = SetAccountDefaultMatchingPoolRequest { matching_pool };
        admin_client.post::<_, EmptyRequestResponse>(&path, body).await?;

        Ok(())
    }
}
