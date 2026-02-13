//! Fetches all open orders managed by the relayer

use renegade_external_api::http::admin::ADMIN_GET_ORDERS_ROUTE;
use renegade_external_api::types::{ApiAdminOrder, GetOrdersAdminResponse};

use crate::{
    RenegadeClientError,
    actions::{MATCHING_POOL_PARAM, PAGE_TOKEN_PARAM},
    client::RenegadeClient,
};

// --- Public Actions --- //
impl RenegadeClient {
    /// Fetches all open orders managed by the relayer.
    ///
    /// This method will paginate through all of the orders across multiple
    /// requests, returning them all.
    pub async fn admin_get_open_orders(&self) -> Result<Vec<ApiAdminOrder>, RenegadeClientError> {
        let admin_relayer_client = self.get_admin_client()?;

        let path = build_admin_get_orders_path(None, None)?;

        let GetOrdersAdminResponse { mut orders, mut next_page_token } =
            admin_relayer_client.get(&path).await?;

        while let Some(page_token) = next_page_token {
            let path = build_admin_get_orders_path(None, Some(page_token))?;

            let response: GetOrdersAdminResponse = admin_relayer_client.get(&path).await?;

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

        let path = build_admin_get_orders_path(Some(&matching_pool), None)?;

        let GetOrdersAdminResponse { mut orders, mut next_page_token } =
            admin_relayer_client.get(&path).await?;

        while let Some(page_token) = next_page_token {
            let path = build_admin_get_orders_path(Some(&matching_pool), Some(page_token))?;

            let response: GetOrdersAdminResponse = admin_relayer_client.get(&path).await?;

            orders.extend(response.orders);
            next_page_token = response.next_page_token;
        }

        Ok(orders)
    }
}

// --- Helpers --- //

/// Builds the request path for the admin get orders endpoint
fn build_admin_get_orders_path(
    matching_pool: Option<&str>,
    page_token: Option<i64>,
) -> Result<String, RenegadeClientError> {
    let mut params: Vec<(&str, String)> = Vec::new();
    if let Some(pool) = matching_pool {
        params.push((MATCHING_POOL_PARAM, pool.to_string()));
    }
    if let Some(token) = page_token {
        params.push((PAGE_TOKEN_PARAM, token.to_string()));
    }

    if params.is_empty() {
        Ok(ADMIN_GET_ORDERS_ROUTE.to_string())
    } else {
        let query_string =
            serde_urlencoded::to_string(&params).map_err(RenegadeClientError::serde)?;
        Ok(format!("{ADMIN_GET_ORDERS_ROUTE}?{query_string}"))
    }
}
