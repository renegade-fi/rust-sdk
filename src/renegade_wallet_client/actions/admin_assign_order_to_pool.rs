//! Admin action to assign an order to a matching pool

use uuid::Uuid;

use crate::{RenegadeClientError, client::RenegadeClient};

impl RenegadeClient {
    /// Assigns an order to a specific matching pool via the admin API.
    ///
    /// This is an admin action that requires the client to be configured with
    /// an admin HMAC key.
    pub async fn admin_assign_order_to_pool(
        &self,
        _order_id: Uuid,
        _matching_pool: String,
    ) -> Result<(), RenegadeClientError> {
        unimplemented!("admin_assign_order_to_pool is not supported in v2")
    }
}
