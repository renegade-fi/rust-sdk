//! An example showing how to fetch token prices for different token pairs

use renegade_sdk::example_utils::build_renegade_client;

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Fetch token prices
    println!("Fetching token prices...");
    let client = build_renegade_client(false /* use_base */)?;
    let prices = client.get_token_prices().await?;

    println!("Found {} token price pairs:", prices.token_prices.len());
    for price in &prices.token_prices {
        println!("{}/{}: {}", price.base_token, price.quote_token, price.price);
    }

    Ok(())
}
