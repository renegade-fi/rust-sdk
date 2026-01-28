//! Admin action to assign an order to a matching pool

use uuid::Uuid;

use crate::{
    RenegadeClientError,
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        ADMIN_ASSIGN_ORDER_TO_POOL_ROUTE,
        admin::ApiAdminOrder,
        request_response::{AdminAssignOrderToPoolRequest, AdminAssignOrderToPoolResponse},
    },
};

impl RenegadeClient {
    /// Assigns an order to a specific matching pool via the admin API.
    ///
    /// This is an admin action that requires the client to be configured with
    /// an admin HMAC key.
    pub async fn admin_assign_order_to_pool(
        &self,
        order_id: Uuid,
        matching_pool: String,
    ) -> Result<ApiAdminOrder, RenegadeClientError> {
        let admin_client = self.get_admin_client()?;

        let path = construct_http_path!(ADMIN_ASSIGN_ORDER_TO_POOL_ROUTE, "order_id" => order_id);
        let request = AdminAssignOrderToPoolRequest { matching_pool };

        let AdminAssignOrderToPoolResponse { order } = admin_client.post(&path, request).await?;

        Ok(order)
    }
}
