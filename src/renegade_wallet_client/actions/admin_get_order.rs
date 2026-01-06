//! Fetch a given order managed by the relayer by its ID

use uuid::Uuid;

use crate::{
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        admin::ApiAdminOrder, request_response::GetOrderAdminResponse, ADMIN_GET_ORDER_ROUTE,
    },
    RenegadeClientError,
};

// --- Public Actions --- //
impl RenegadeClient {
    /// Look up an order by its ID
    pub async fn admin_get_order(
        &self,
        order_id: Uuid,
    ) -> Result<ApiAdminOrder, RenegadeClientError> {
        let admin_relayer_client = self.get_admin_client()?;

        let path = construct_http_path!(ADMIN_GET_ORDER_ROUTE, "order_id" => order_id);

        let GetOrderAdminResponse { order } = admin_relayer_client.get(&path).await?;
        Ok(order)
    }
}
