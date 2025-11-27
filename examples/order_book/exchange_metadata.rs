//! Example of getting exchange metadata
use renegade_sdk::example_utils::build_renegade_client;

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Get the external match client
    let client = build_renegade_client(false /* use_base */).unwrap();

    // Get exchange metadata
    println!("Fetching exchange metadata...");
    let metadata = client.get_exchange_metadata().await?;

    println!("\nExchange Metadata:");
    println!("Chain ID: {}", metadata.chain_id);
    println!("Settlement Contract Address: {}", metadata.settlement_contract_address);
    println!("\nSupported Tokens ({}):", metadata.supported_tokens.len());

    // Print token information
    for (i, token) in metadata.supported_tokens.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, token.symbol, token.address);
    }

    Ok(())
}
