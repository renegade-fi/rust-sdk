//! Looks up an order by its ID in the relayer

use uuid::Uuid;

use crate::{
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        orders::ApiOrder, request_response::GetOrderByIdResponse, GET_ORDER_BY_ID_ROUTE,
    },
    RenegadeClientError,
};

// --- Public Actions --- //
impl RenegadeClient {
    /// Look up an order by its ID
    pub async fn get_order(&self, order_id: Uuid) -> Result<ApiOrder, RenegadeClientError> {
        let path = construct_http_path!(GET_ORDER_BY_ID_ROUTE, "account_id" => self.get_account_id(), "order_id" => order_id);

        let GetOrderByIdResponse { order } = self.relayer_client.get(&path).await?;
        Ok(order)
    }
}
