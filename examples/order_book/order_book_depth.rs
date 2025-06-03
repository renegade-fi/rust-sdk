//! Example of getting the order book depth for a token
use renegade_sdk::{example_utils::build_renegade_client, ExternalMatchClient};

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Get the external match client
    let api_key = std::env::var("EXTERNAL_MATCH_KEY").unwrap();
    let api_secret = std::env::var("EXTERNAL_MATCH_SECRET").unwrap();
    let client = build_renegade_client(false /* use_base */).unwrap();

    // Fetch supported tokens
    println!("Fetching supported tokens...");
    let tokens = client.get_supported_tokens().await?;

    // Find and print WETH details
    let mut weth_address = "0x".to_string();
    for token in tokens.tokens {
        if token.symbol.to_uppercase() == "WETH" {
            println!("\nFound WETH!");
            println!("Address: {}", token.address);
            weth_address = token.address;
        }
    }

    // Get the order book depth for WETH
    println!("Fetching order book depth for WETH...");
    let depth = client.get_order_book_depth(&weth_address).await?;
    println!("Current price: ${:.2}", depth.price);
    println!("Timestamp: {}", depth.timestamp);
    println!("\nBuy side:");
    println!("  Total quantity: {}", depth.buy.total_quantity);
    println!("  Total quantity (USD): ${:.2}", depth.buy.total_quantity_usd);
    println!("\nSell side:");
    println!("  Total quantity: {}", depth.sell.total_quantity);
    println!("  Total quantity (USD): ${:.2}", depth.sell.total_quantity_usd);

    Ok(())
}
