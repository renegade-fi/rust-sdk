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
    let client = build_renegade_client()?;
    let order = ExternalOrderBuilder::new()
        .base_mint(BASE_MINT)
        .quote_mint(QUOTE_MINT)
        .quote_amount(30_000_000) // $30 USDC
        .min_fill_size(1_000_000) // $1 USDC
        .side(OrderSide::Sell)
        .build()
        .unwrap();

    fetch_quote_and_execute_malleable(&client, order, &signer).await?;
    Ok(())
}

/// Fetch a quote from the external api and execute a malleable match
///
/// Malleable matches allow the exact swap amount to be determined at settlement
/// time within a predefined range, offering more flexibility than standard
/// matches.
async fn fetch_quote_and_execute_malleable(
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

    // Assemble the quote into a malleable bundle
    println!("Assembling malleable quote...");
    let resp = match client.assemble_malleable_quote(quote).await? {
        Some(resp) => resp,
        None => eyre::bail!("No malleable bundle found"),
    };

    // Print information about the malleable match
    let bundle = &resp.match_bundle;
    let result = &bundle.match_result;

    println!("Malleable match details:");
    println!("  Direction: {:?}", result.direction);
    println!("  Price: {}", result.price);
    println!("  Min base amount: {}", result.min_base_amount);
    println!("  Max base amount: {}", result.max_base_amount);

    println!("Fee rates:");
    println!("  Relayer fee rate: {}", bundle.fee_rates.relayer_fee_rate);
    println!("  Protocol fee rate: {}", bundle.fee_rates.protocol_fee_rate);

    // Execute the bundle
    println!("Executing malleable match bundle...");
    execute_bundle(wallet, bundle.settlement_tx.clone()).await
}
