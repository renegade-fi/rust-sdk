//! Looks up an order by its ID in the relayer

use renegade_api::{
    http::wallet::{GetOrderByIdResponse, GET_ORDER_BY_ID_ROUTE},
    types::ApiOrder,
};
use uuid::Uuid;

use crate::{actions::construct_http_path, client::RenegadeClient, RenegadeClientError};

impl RenegadeClient {
    /// Look up an order by its ID
    ///
    /// This method will return the order if it exists in the relayer's current
    /// view of the wallet.
    /// Note that this is *not* the back-of-queue view of the wallet. See
    /// [`RenegadeClient::get_wallet`] for more details.
    pub async fn get_order(&self, order_id: Uuid) -> Result<ApiOrder, RenegadeClientError> {
        let wallet_id = self.secrets.wallet_id;
        let path = construct_http_path!(GET_ORDER_BY_ID_ROUTE, "wallet_id" => wallet_id, "order_id" => order_id);
        let response: GetOrderByIdResponse = self.get_relayer(&path).await?;
        Ok(response.order)
    }
}
