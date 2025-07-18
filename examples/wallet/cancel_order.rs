//! Example of canceling an order in the wallet

use alloy::signers::local::PrivateKeySigner;
use renegade_sdk::client::RenegadeClient;
use std::str::FromStr;
use uuid::Uuid;

/// The private key to use for the wallet
const PKEY: &str = env!("PKEY");

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Read the private key from the environment variable
    let private_key = PrivateKeySigner::from_str(PKEY)?;
    let eth_address = private_key.address();
    println!("Ethereum address: {:#x}", eth_address);

    // Create the Renegade client for Arbitrum Sepolia
    let renegade_client = RenegadeClient::new_arbitrum_sepolia(&private_key)?;

    // Get the wallet to see existing orders
    let wallet = renegade_client.get_wallet().await?;
    if wallet.orders.is_empty() {
        println!("No orders found in wallet to cancel");
        return Ok(());
    }

    // Get the first order ID from the wallet
    let order_id = wallet.orders[0].id;
    println!("Canceling order with ID: {}", order_id);
    match renegade_client.cancel_order(order_id).await {
        Ok(()) => println!("Successfully canceled order!"),
        Err(e) => println!("Failed to cancel order: {e}"),
    }

    Ok(())
}
