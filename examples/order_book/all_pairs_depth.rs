//! Example of getting the order book depth for all supported pairs
use renegade_sdk::example_utils::build_renegade_client;

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Get the external match client
    let client = build_renegade_client(false /* use_base */).unwrap();

    // Get the order book depth for all pairs
    println!("Fetching order book depth for all supported pairs...");
    let all_pairs_depth = client.get_order_book_depth_all_pairs().await?;

    println!("Found {} supported pairs:\n", all_pairs_depth.pairs.len());

    // Print depth information for each pair
    for (i, pair) in all_pairs_depth.pairs.iter().enumerate() {
        println!("{}. Token Address: {}", i + 1, pair.address);
        println!("   Current price: ${:.2}", pair.price);
        println!("   Timestamp: {}", pair.timestamp);
        println!("   Buy side:");
        println!("     Total quantity: {}", pair.buy.total_quantity);
        println!("     Total quantity (USD): ${:.2}", pair.buy.total_quantity_usd);
        println!("   Sell side:");
        println!("     Total quantity: {}", pair.sell.total_quantity);
        println!("     Total quantity (USD): ${:.2}", pair.sell.total_quantity_usd);
        println!();
    }

    Ok(())
}
