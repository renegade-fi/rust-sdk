//! Fetches all orders for a given account (admin)

use uuid::Uuid;

use crate::{
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        admin::ApiAdminOrder,
        request_response::{GetAccountOrdersAdminQueryParameters, GetAccountOrdersAdminResponse},
        ADMIN_GET_ACCOUNT_ORDERS_ROUTE,
    },
    RenegadeClientError,
};

// --- Public Actions --- //
impl RenegadeClient {
    /// Fetches all orders for the given account (admin).
    ///
    /// This method will paginate through all of the account's orders across
    /// multiple requests, returning them all.
    pub async fn admin_get_account_orders(
        &self,
        account_id: Uuid,
    ) -> Result<Vec<ApiAdminOrder>, RenegadeClientError> {
        let admin_relayer_client = self.get_admin_client()?;

        let mut query_params: GetAccountOrdersAdminQueryParameters = Default::default();
        let path = Self::build_admin_get_account_orders_request_path(account_id, &query_params)?;

        let GetAccountOrdersAdminResponse { mut orders, mut next_page_token } =
            admin_relayer_client.get(&path).await?;

        while let Some(page_token) = next_page_token {
            query_params.page_token = Some(page_token);
            let path =
                Self::build_admin_get_account_orders_request_path(account_id, &query_params)?;

            let response: GetAccountOrdersAdminResponse = admin_relayer_client.get(&path).await?;

            orders.extend(response.orders);
            next_page_token = response.next_page_token;
        }

        Ok(orders)
    }
}

// --- Private Helpers --- //
impl RenegadeClient {
    /// Builds the request path for the admin get account orders endpoint
    fn build_admin_get_account_orders_request_path(
        account_id: Uuid,
        query_params: &GetAccountOrdersAdminQueryParameters,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(ADMIN_GET_ACCOUNT_ORDERS_ROUTE, "account_id" => account_id);
        let query_string =
            serde_urlencoded::to_string(query_params).map_err(RenegadeClientError::serde)?;

        Ok(format!("{path}?{query_string}"))
    }
}
