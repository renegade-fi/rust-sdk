//! Looks up an order by its ID in the relayer

use renegade_external_api::{
    http::order::{GET_ORDER_BY_ID_ROUTE, GetOrderByIdResponse},
    types::{ApiOrder, OrderAuth},
};
use uuid::Uuid;

use crate::{RenegadeClientError, actions::construct_http_path, client::RenegadeClient};

// --- Public Actions --- //
impl RenegadeClient {
    /// Look up an order by its ID
    pub async fn get_order(&self, order_id: Uuid) -> Result<ApiOrder, RenegadeClientError> {
        let (order, _auth) = self.get_order_with_auth(order_id).await?;
        Ok(order)
    }
}

// --- Private Helpers --- //
impl RenegadeClient {
    /// Gets the order and its auth from the relayer
    pub(crate) async fn get_order_with_auth(
        &self,
        order_id: Uuid,
    ) -> Result<(ApiOrder, OrderAuth), RenegadeClientError> {
        let path = construct_http_path!(GET_ORDER_BY_ID_ROUTE, "account_id" => self.get_account_id(), "order_id" => order_id);
        let GetOrderByIdResponse { order, auth } = self.relayer_client.get(&path).await?;
        Ok((order, auth))
    }
}
