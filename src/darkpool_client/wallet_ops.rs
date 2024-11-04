//! Operations with a wallet

use eyre::Result;
use num_bigint::BigUint;
use renegade_api::{
    http::wallet::{
        CancelOrderRequest, CancelOrderResponse, CreateOrderRequest, CreateOrderResponse,
        CreateWalletRequest, CreateWalletResponse, GetWalletResponse, BACK_OF_QUEUE_WALLET_ROUTE,
        CANCEL_ORDER_ROUTE, CREATE_WALLET_ROUTE, GET_WALLET_ROUTE, WALLET_ORDERS_ROUTE,
    },
    types::{ApiOrder, ApiWallet},
};
use renegade_common::types::wallet::{Order, OrderIdentifier, Wallet};

use crate::util::construct_http_path;

use super::{auth::reblind_and_authorize_update, DarkpoolClient};

impl DarkpoolClient {
    // --- Getters --- //

    /// Get the wallet from the darkpool
    pub async fn get_wallet(&self) -> Result<Wallet> {
        let id = &self.wallet_secrets.wallet_id;
        let path = construct_http_path!(GET_WALLET_ROUTE, "wallet_id" => id);
        let resp: GetWalletResponse = self.http_client.get(&path).await?;
        let wallet: Wallet = resp.wallet.try_into().map_err(|e: String| eyre::eyre!(e))?;
        Ok(wallet)
    }

    /// Get the wallet from the relayer, applying all pending tasks to the
    /// wallet
    pub async fn get_back_of_queue_wallet(&self) -> Result<Wallet> {
        let id = &self.wallet_secrets.wallet_id;
        let path = construct_http_path!(BACK_OF_QUEUE_WALLET_ROUTE, "wallet_id" => id);
        let resp: GetWalletResponse = self.http_client.get(&path).await?;
        let wallet: Wallet = resp.wallet.try_into().map_err(|e: String| eyre::eyre!(e))?;
        Ok(wallet)
    }

    // --- Setters --- //

    /// Create a new wallet in the darkpool
    pub async fn create_wallet(&self) -> Result<Wallet> {
        let secrets = &self.wallet_secrets;
        let new_wallet = Wallet::new_empty_wallet(
            secrets.wallet_id,
            secrets.blinder_seed,
            secrets.share_seed,
            secrets.keychain.clone(),
        );

        let wallet: ApiWallet = new_wallet.clone().into();
        let blinder_seed: BigUint = secrets.blinder_seed.into();
        let req = CreateWalletRequest { wallet, blinder_seed };

        let _resp: CreateWalletResponse = self.http_client.post(CREATE_WALLET_ROUTE, req).await?;
        Ok(new_wallet)
    }

    /// Create an order in the wallet
    pub async fn create_order(&self, order: Order) -> Result<Wallet> {
        // Update the wallet and sign the new shares
        let mut wallet = self.get_back_of_queue_wallet().await?;
        let order_id = OrderIdentifier::new_v4();
        wallet.add_order(order_id, order.clone()).map_err(|e| eyre::eyre!(e))?;
        let update_auth = reblind_and_authorize_update(&mut wallet)?;

        let id = &self.wallet_secrets.wallet_id;
        let path = construct_http_path!(WALLET_ORDERS_ROUTE, "wallet_id" => id);

        // Create the order
        let order = ApiOrder::from((order_id, order));
        let req = CreateOrderRequest { order, update_auth };
        let _resp: CreateOrderResponse = self.http_client.post(&path, req).await?;

        let wallet = self.get_back_of_queue_wallet().await?;
        Ok(wallet)
    }

    /// Cancel an order in the wallet
    pub async fn cancel_order(&self, order_id: OrderIdentifier) -> Result<Wallet> {
        // Remove the order from the wallet
        let mut wallet = self.get_back_of_queue_wallet().await?;
        if wallet.remove_order(&order_id).is_none() {
            return Err(eyre::eyre!("order not found"));
        }
        let update_auth = reblind_and_authorize_update(&mut wallet)?;

        // Cancel the order
        let id = &self.wallet_secrets.wallet_id;
        let path =
            construct_http_path!(CANCEL_ORDER_ROUTE, "wallet_id" => id, "order_id" => order_id);

        let req = CancelOrderRequest { update_auth };
        let _resp: CancelOrderResponse = self.http_client.post(&path, req).await?;
        let wallet = self.get_back_of_queue_wallet().await?;
        Ok(wallet)
    }
}
