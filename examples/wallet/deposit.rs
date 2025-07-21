//! Example of depositing funds into the wallet

use alloy::signers::local::PrivateKeySigner;
use renegade_sdk::client::RenegadeClient;
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

    // Deposit 1 WETH into the wallet
    let token_mint = WETH_ADDRESS;
    let amount = 10_u128.pow(18); // 1 WETH

    // Deposit the funds into the wallet
    match renegade_client.deposit(token_mint, amount, &private_key).await {
        Ok(_) => {
            println!("Successfully deposited {amount} units of token {token_mint} into wallet!")
        },
        Err(e) => println!("Failed to deposit: {e}"),
    }

    Ok(())
}
