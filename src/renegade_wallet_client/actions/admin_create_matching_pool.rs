//! Admin action to create a matching pool

use renegade_external_api::EmptyRequestResponse;

use crate::{
    client::RenegadeClient,
    renegade_api_types::{
        request_response::AdminCreateMatchingPoolRequest, ADMIN_CREATE_MATCHING_POOL_ROUTE,
    },
    RenegadeClientError,
};

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

        let request = AdminCreateMatchingPoolRequest { matching_pool };

        admin_client
            .post::<_, EmptyRequestResponse>(ADMIN_CREATE_MATCHING_POOL_ROUTE, request)
            .await?;

        Ok(())
    }
}
