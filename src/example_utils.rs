//! Utilities for the examples

use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use alloy_rpc_types_eth::TransactionRequest;
use std::str::FromStr;
use std::sync::Arc;
use url::Url;

use crate::ExternalMatchClient;

/// The RPC URL to use
const RPC_URL: &str = env!("RPC_URL");

/// The middleware type
pub type Wallet = Arc<dyn Provider>;

/// Build a Renegade client from environment variables
pub fn build_renegade_client() -> Result<ExternalMatchClient, eyre::Error> {
    let api_key = std::env::var("EXTERNAL_MATCH_KEY").unwrap();
    let api_secret = std::env::var("EXTERNAL_MATCH_SECRET").unwrap();
    let client = ExternalMatchClient::new_arbitrum_sepolia_client(&api_key, &api_secret).unwrap();
    Ok(client)
}

/// Get a wallet from a private key environment variable
pub async fn get_signer() -> Result<Wallet, eyre::Error> {
    let url = Url::parse(RPC_URL).unwrap();
    let pkey = std::env::var("PKEY").unwrap();
    let wallet = PrivateKeySigner::from_str(&pkey).unwrap();
    let provider = ProviderBuilder::new().wallet(wallet).connect_http(url);

    Ok(Arc::new(provider))
}

/// Execute a bundle directly
pub async fn execute_bundle(wallet: &Wallet, tx: TransactionRequest) -> Result<(), eyre::Error> {
    println!("Submitting bundle...\n");
    let hash = wallet.send_transaction(tx).await?.watch().await?;
    println!("Successfully submitted transaction: {:#x}", hash);
    Ok(())
}
