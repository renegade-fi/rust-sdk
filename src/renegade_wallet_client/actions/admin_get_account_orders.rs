//! Fetches all orders for a given account (admin)

use renegade_external_api::http::admin::ADMIN_GET_ACCOUNT_ORDERS_ROUTE;
use renegade_external_api::types::{ApiAdminOrder, GetOrdersAdminResponse};
use uuid::Uuid;

use crate::{
    RenegadeClientError,
    actions::{PAGE_TOKEN_PARAM, construct_http_path},
    client::RenegadeClient,
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

        let path = Self::build_admin_get_account_orders_request_path(account_id, None)?;

        let GetOrdersAdminResponse { mut orders, mut next_page_token } =
            admin_relayer_client.get(&path).await?;

        while let Some(page_token) = next_page_token {
            let path =
                Self::build_admin_get_account_orders_request_path(account_id, Some(page_token))?;

            let response: GetOrdersAdminResponse = admin_relayer_client.get(&path).await?;

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
        page_token: Option<i64>,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(ADMIN_GET_ACCOUNT_ORDERS_ROUTE, "account_id" => account_id);

        if let Some(token) = page_token {
            let query_string =
                serde_urlencoded::to_string(&[(PAGE_TOKEN_PARAM, token.to_string())])
                    .map_err(RenegadeClientError::serde)?;
            Ok(format!("{path}?{query_string}"))
        } else {
            Ok(path)
        }
    }
}
