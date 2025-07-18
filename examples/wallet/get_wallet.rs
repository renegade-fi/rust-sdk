//! Example of getting wallet state from the relayer

use alloy::signers::local::PrivateKeySigner;
use eyre::Result;
use renegade_sdk::client::RenegadeClient;
use renegade_utils::hex::biguint_to_hex_string;
use std::str::FromStr;

/// The private key to use for the wallet
const PKEY: &str = env!("PKEY");

#[tokio::main]
async fn main() -> Result<()> {
    // Read the private key from the environment variable
    let private_key = PrivateKeySigner::from_str(PKEY)?;
    let eth_address = private_key.address();
    println!("Ethereum address: {:#x}", eth_address);

    // Create the Renegade client for Arbitrum Sepolia
    let renegade_client = RenegadeClient::new_arbitrum_sepolia(&private_key)?;
    println!("\nGetting wallet state from relayer...");

    // Get the wallet state from the relayer
    let wallet = renegade_client.get_wallet().await?;
    println!("Wallet ID: {}", renegade_client.secrets.wallet_id);
    println!("Wallet Orders:");
    for order in wallet.orders.iter().filter(|o| o.amount > 0) {
        let base_mint_hex = biguint_to_hex_string(&order.base_mint);
        println!("\t - {}: {} {} {}", order.id, order.side, order.amount, base_mint_hex)
    }

    println!("Wallet Balances:");
    for balance in wallet.balances.iter().filter(|b| b.amount > 0) {
        let balance_mint = biguint_to_hex_string(&balance.mint);
        println!("\t - {}: {}", balance_mint, balance.amount)
    }

    Ok(())
}
