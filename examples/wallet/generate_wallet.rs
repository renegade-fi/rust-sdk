//! Example of generating a random Ethereum key and creating a Renegade wallet
//! client

use alloy::signers::local::PrivateKeySigner;
use renegade_sdk::client::RenegadeClient;

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    println!("Generating random Ethereum private key...");

    // Generate a random private key
    let private_key = PrivateKeySigner::random();
    let eth_address = private_key.address();
    println!("Ethereum address: {:#x}", eth_address);
    println!("Creating Renegade wallet client for Arbitrum Sepolia...");

    // Create the Renegade client for Arbitrum Sepolia
    let renegade_client = RenegadeClient::new_arbitrum_sepolia(&private_key)?;
    println!("Successfully created Renegade wallet!");
    println!("Wallet ID: {}", renegade_client.secrets.wallet_id);
    println!("Wallet generation complete!");

    Ok(())
}
