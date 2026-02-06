//! Admin action to create a matching pool

use renegade_external_api::EmptyRequestResponse;
use renegade_external_api::http::admin::ADMIN_MATCHING_POOL_CREATE_ROUTE;

use crate::{RenegadeClientError, actions::construct_http_path, client::RenegadeClient};

impl RenegadeClient {
    /// Creates a new matching pool via the admin API.
    ///
    /// Orders can only be matched with other orders in the same matching pool.
    /// If the specified matching pool already exists, this is a no-op.
    ///
    /// This is an admin action that requires the client to be configured with
    /// an admin HMAC key.
    pub async fn admin_create_matching_pool(
        &self,
        matching_pool: String,
    ) -> Result<(), RenegadeClientError> {
        let admin_client = self.get_admin_client()?;

        let path = construct_http_path!(ADMIN_MATCHING_POOL_CREATE_ROUTE, "matching_pool" => matching_pool);

        admin_client.post::<_, EmptyRequestResponse>(&path, EmptyRequestResponse {}).await?;

        Ok(())
    }
}
