//! An example showing how to fetch supported tokens and find specific token
//! addresses

use renegade_sdk::ExternalMatchClient;

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Get the external match client
    let api_key = std::env::var("EXTERNAL_MATCH_KEY").unwrap();
    let api_secret = std::env::var("EXTERNAL_MATCH_SECRET").unwrap();
    let client = ExternalMatchClient::new_arbitrum_sepolia_client(&api_key, &api_secret).unwrap();

    // Fetch supported tokens
    println!("Fetching supported tokens...");
    let tokens = client.get_supported_tokens().await?;

    // Find and print WETH details
    for token in tokens.tokens {
        if token.symbol.to_uppercase() == "WETH" {
            println!("\nFound WETH!");
            println!("Address: {}", token.address);
        }
    }

    Ok(())
}
