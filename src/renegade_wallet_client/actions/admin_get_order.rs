//! Fetch a given order managed by the relayer by its ID

use renegade_external_api::http::admin::ADMIN_GET_ORDER_BY_ID_ROUTE;
use renegade_external_api::types::{ApiAdminOrder, GetOrderAdminResponse};
use uuid::Uuid;

use crate::{RenegadeClientError, actions::construct_http_path, client::RenegadeClient};

// --- Public Actions --- //
impl RenegadeClient {
    /// Look up an order by its ID
    pub async fn admin_get_order(
        &self,
        order_id: Uuid,
    ) -> Result<ApiAdminOrder, RenegadeClientError> {
        let admin_relayer_client = self.get_admin_client()?;

        let path = construct_http_path!(ADMIN_GET_ORDER_BY_ID_ROUTE, "order_id" => order_id);

        let GetOrderAdminResponse { order } = admin_relayer_client.get(&path).await?;
        Ok(order)
    }
}
