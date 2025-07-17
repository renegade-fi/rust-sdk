//! Example of creating a wallet in the darkpool

use alloy::signers::local::PrivateKeySigner;
use renegade_sdk::client::RenegadeClient;
use std::str::FromStr;

/// The private key to use for the wallet
const PKEY: &str = env!("PKEY");

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Read the private key from the environment variable
    let private_key = PrivateKeySigner::from_str(PKEY)?;
    let eth_address = private_key.address();
    println!("Ethereum address: {:#x}", eth_address);
    println!("Creating Renegade wallet client for Arbitrum Sepolia...");

    // Create the Renegade client for Arbitrum Sepolia
    let renegade_client = RenegadeClient::new_arbitrum_sepolia(&private_key)?;
    println!("Wallet ID: {}", renegade_client.secrets.wallet_id);
    println!("\nCreating wallet in darkpool...");

    // Create the wallet in the darkpool
    match renegade_client.create_wallet().await {
        Ok(()) => println!("Successfully created wallet in darkpool!"),
        Err(e) => println!("Failed to create wallet: {e}"),
    }

    Ok(())
}
