//! Operations with a wallet

use eyre::Result;
use num_bigint::BigUint;
use renegade_api::{
    http::wallet::{
        CreateWalletRequest, CreateWalletResponse, GetWalletResponse, CREATE_WALLET_ROUTE,
        GET_WALLET_ROUTE,
    },
    types::ApiWallet,
};
use renegade_common::types::wallet::Wallet;

use crate::util::construct_http_path;

use super::DarkpoolClient;

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
}
