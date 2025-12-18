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

impl RenegadeClient {
    /// Look up an order by its ID
    ///
    /// This method will return the order if it exists in the relayer's current
    /// view of the account.
    pub async fn get_order(&self, order_id: Uuid) -> Result<ApiOrder, RenegadeClientError> {
        let account_id = self.secrets.account_id;
        let path = construct_http_path!(GET_ORDER_BY_ID_ROUTE, "account_id" => account_id, "order_id" => order_id);

        let response: GetOrderByIdResponse = self.relayer_client.get(&path).await?;
        Ok(response.order)
    }
}
