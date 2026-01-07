//! Fetches all orders in the account

use crate::{
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        orders::ApiOrder,
        request_response::{GetOrdersQueryParameters, GetOrdersResponse},
        GET_ORDERS_ROUTE,
    },
    RenegadeClientError,
};

// --- Public Actions --- //
impl RenegadeClient {
    /// Fetches all orders in the account, optionally including historic
    /// (inactive) orders.
    ///
    /// This method will paginate through all of the account's orders across
    /// multiple requests, returning them all.
    pub async fn get_orders(
        &self,
        include_historic_orders: bool,
    ) -> Result<Vec<ApiOrder>, RenegadeClientError> {
        let mut query_params = GetOrdersQueryParameters {
            include_historic_orders: Some(include_historic_orders),
            ..Default::default()
        };
        let path = self.build_get_orders_request_path(&query_params)?;

        let GetOrdersResponse { mut orders, mut next_page_token } =
            self.relayer_client.get(&path).await?;

        while let Some(page_token) = next_page_token {
            query_params.page_token = Some(page_token);
            let path = self.build_get_orders_request_path(&query_params)?;

            let response: GetOrdersResponse = self.relayer_client.get(&path).await?;

            orders.extend(response.orders);
            next_page_token = response.next_page_token;
        }

        Ok(orders)
    }
}

// --- Private Helpers --- //
impl RenegadeClient {
    /// Builds the request path for the get orders endpoint
    fn build_get_orders_request_path(
        &self,
        query_params: &GetOrdersQueryParameters,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(GET_ORDERS_ROUTE, "account_id" => self.get_account_id());
        let query_string =
            serde_urlencoded::to_string(query_params).map_err(RenegadeClientError::serde)?;

        Ok(format!("{path}?{query_string}"))
    }
}
