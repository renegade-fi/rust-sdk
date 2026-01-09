//! Fetches all open orders managed by the relayer

use crate::{
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        admin::ApiAdminOrder,
        request_response::{GetOpenOrdersAdminQueryParameters, GetOpenOrdersAdminResponse},
        ADMIN_GET_OPEN_ORDERS_ROUTE,
    },
    RenegadeClientError,
};

// --- Public Actions --- //
impl RenegadeClient {
    /// Fetches all open orders managed by the relayer.
    ///
    /// This method will paginate through all of the orders across multiple
    /// requests, returning them all.
    pub async fn admin_get_open_orders(&self) -> Result<Vec<ApiAdminOrder>, RenegadeClientError> {
        let admin_relayer_client = self.get_admin_client()?;

        let mut query_params: GetOpenOrdersAdminQueryParameters = Default::default();
        let path = self.build_admin_get_open_orders_request_path(&query_params)?;

        let GetOpenOrdersAdminResponse { mut orders, mut next_page_token } =
            admin_relayer_client.get(&path).await?;

        while let Some(page_token) = next_page_token {
            query_params.page_token = Some(page_token);
            let path = self.build_admin_get_open_orders_request_path(&query_params)?;

            let response: GetOpenOrdersAdminResponse = admin_relayer_client.get(&path).await?;

            orders.extend(response.orders);
            next_page_token = response.next_page_token;
        }

        Ok(orders)
    }

    /// Fetches all open orders managed by the relayer in the given matching
    /// pool.
    ///
    /// This method will paginate through all of the orders across multiple
    /// requests, returning them all.
    pub async fn admin_get_open_orders_in_matching_pool(
        &self,
        matching_pool: String,
    ) -> Result<Vec<ApiAdminOrder>, RenegadeClientError> {
        let admin_relayer_client = self.get_admin_client()?;

        let mut query_params = GetOpenOrdersAdminQueryParameters {
            matching_pool: Some(matching_pool),
            ..Default::default()
        };
        let path = self.build_admin_get_open_orders_request_path(&query_params)?;

        let GetOpenOrdersAdminResponse { mut orders, mut next_page_token } =
            admin_relayer_client.get(&path).await?;

        while let Some(page_token) = next_page_token {
            query_params.page_token = Some(page_token);
            let path = self.build_admin_get_open_orders_request_path(&query_params)?;

            let response: GetOpenOrdersAdminResponse = admin_relayer_client.get(&path).await?;

            orders.extend(response.orders);
            next_page_token = response.next_page_token;
        }

        Ok(orders)
    }
}

// --- Private Helpers --- //
impl RenegadeClient {
    /// Builds the request path for the get open orders endpoint
    fn build_admin_get_open_orders_request_path(
        &self,
        query_params: &GetOpenOrdersAdminQueryParameters,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(ADMIN_GET_OPEN_ORDERS_ROUTE, "account_id" => self.get_account_id());
        let query_string =
            serde_urlencoded::to_string(query_params).map_err(RenegadeClientError::serde)?;

        Ok(format!("{path}?{query_string}"))
    }
}
