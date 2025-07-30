//! Example demonstrating how to use exact output amounts in external match
//! orders
//!
//! This example shows how to create orders using the `exact_base_output` and
//! `exact_quote_output` fields, which allow you to specify precisely how much
//! of a token you want to receive from the trade.

use renegade_sdk::example_utils::{build_renegade_client, execute_bundle, get_signer, Wallet};
use renegade_sdk::{
    types::{ExternalOrder, OrderSide},
    ExternalMatchClient, ExternalOrderBuilder,
};

/// Testnet wETH
const BASE_MINT: &str = "0xc3414a7ef14aaaa9c4522dfc00a4e66e74e9c25a";
/// Testnet USDC
const QUOTE_MINT: &str = "0xdf8d259c04020562717557f2b5a3cf28e92707d1";

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Get wallet from private key
    let signer = get_signer().await?;

    // Get the external match client
    let client = build_renegade_client(false /* use_base */)?;

    // Using exact_quote_output for a buy order
    // When buying wETH with USDC, this means we want to spend exactly 30 USDC
    // and receive whatever amount of wETH that buys at current market price.
    // This differs from quote_amount which would specify how much we're willing to
    // spend.
    let order = ExternalOrderBuilder::new()
        .base_mint(BASE_MINT)
        .quote_mint(QUOTE_MINT)
        .exact_quote_output(30_000_000) // Exactly $30 USDC to spend
        .side(OrderSide::Buy)
        .build()
        .unwrap();

    println!("=== Buy Order with Exact Quote Output ===");
    println!("Order: {:?}", order);
    fetch_quote_and_execute(&client, order, &signer).await?;

    Ok(())
}

/// Fetch a quote from the external api and print it
async fn fetch_quote_and_execute(
    client: &ExternalMatchClient,
    order: ExternalOrder,
    wallet: &Wallet,
) -> Result<(), eyre::Error> {
    // Fetch a quote from the relayer
    println!("Fetching quote...");
    let res = client.request_quote(order).await?;
    let quote = match res {
        Some(quote) => quote,
        None => eyre::bail!("No quote found"),
    };

    // Assemble the quote into a bundle
    println!("Assembling quote...");
    let resp = match client.assemble_quote(quote).await? {
        Some(resp) => resp,
        None => eyre::bail!("No bundle found"),
    };

    execute_bundle(wallet, resp.match_bundle.settlement_tx).await
}
