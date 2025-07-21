//! Example of placing an order in the wallet and waiting for it to complete

use alloy::signers::local::PrivateKeySigner;
use renegade_circuit_types::order::OrderSide;
use renegade_sdk::{
    actions::place_order::OrderBuilder, api_types::FixedPoint, client::RenegadeClient,
};
use std::str::FromStr;

/// WETH address on arbitrum sepolia
const WETH_ADDRESS: &str = "0xc3414a7ef14aaaa9c4522dfc00a4e66e74e9c25a";
/// USDC address on arbitrum sepolia  
const USDC_ADDRESS: &str = "0xdf8d259c04020562717557f2b5a3cf28e92707d1";

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

    // Create a sample order (selling WETH for USDC)
    let amount = 10_u128.pow(18); // 1 WETH
    let order = OrderBuilder::new()
        .with_base_mint(WETH_ADDRESS)?
        .with_quote_mint(USDC_ADDRESS)?
        .with_side(OrderSide::Sell)
        .with_amount(amount)
        .build()?;

    // Place the order in the wallet and get a task waiter
    println!("Placing order and waiting for task completion...");
    let mut task_waiter = renegade_client.place_order(order.clone()).await?;

    // Await the task completion using the watch_task method
    match task_waiter.watch_task().await {
        Ok(()) => println!("Order placement task completed successfully!"),
        Err(e) => println!("Order placement task failed: {e}"),
    }

    Ok(())
}
