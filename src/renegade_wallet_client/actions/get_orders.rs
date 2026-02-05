//! Fetches all orders in the account

use renegade_external_api::{
    http::order::{GET_ORDERS_ROUTE, GetOrdersResponse},
    types::ApiOrder,
};

use crate::{
    RenegadeClientError,
    actions::{INCLUDE_HISTORIC_ORDERS_PARAM, PAGE_TOKEN_PARAM, construct_http_path},
    client::RenegadeClient,
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
        let path = self.build_get_orders_request_path(include_historic_orders, None)?;

        let GetOrdersResponse { mut orders, mut next_page_token } =
            self.relayer_client.get(&path).await?;

        while let Some(page_token) = next_page_token {
            let path =
                self.build_get_orders_request_path(include_historic_orders, Some(page_token))?;

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
        include_historic_orders: bool,
        page_token: Option<i64>,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(GET_ORDERS_ROUTE, "account_id" => self.get_account_id());

        let mut params = vec![(INCLUDE_HISTORIC_ORDERS_PARAM, include_historic_orders.to_string())];
        if let Some(token) = page_token {
            params.push((PAGE_TOKEN_PARAM, token.to_string()));
        }

        let query_string =
            serde_urlencoded::to_string(&params).map_err(RenegadeClientError::serde)?;

        Ok(format!("{path}?{query_string}"))
    }
}
