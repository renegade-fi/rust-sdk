//! Buy with exact base output: receive exactly N base tokens.
//!
//! "I'm buying wETH and want to receive exactly this many."

use renegade_sdk::example_utils::{Wallet, build_renegade_client, execute_bundle, get_signer};
use renegade_sdk::{
    ExternalMatchClient, ExternalOrderBuilder,
    types::{ExternalOrder, OrderSide},
};

/// Testnet wETH
const BASE_MINT: &str = "0xc3414a7ef14aaaa9c4522dfc00a4e66e74e9c25a";
/// Testnet USDC
const QUOTE_MINT: &str = "0xdf8d259c04020562717557f2b5a3cf28e92707d1";

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    let signer = get_signer().await?;
    let client = build_renegade_client(false /* use_base */)?;

    let order = ExternalOrderBuilder::new()
        .base_mint(BASE_MINT)
        .quote_mint(QUOTE_MINT)
        .exact_base_output(10_000_000_000_000_000) // 0.01 wETH (18 decimals)
        .side(OrderSide::Buy)
        .build()
        .unwrap();

    println!("=== Buy + exact_base_output (exact receive) ===");
    println!("Goal: receive exactly 0.01 wETH, pay whatever USDC is needed");
    fetch_quote_and_execute(&client, order, &signer).await
}

async fn fetch_quote_and_execute(
    client: &ExternalMatchClient,
    order: ExternalOrder,
    wallet: &Wallet,
) -> Result<(), eyre::Error> {
    println!("Fetching quote...");
    let quote = client.request_quote(order).await?.ok_or_else(|| eyre::eyre!("No quote found"))?;

    println!("Assembling quote...");
    let resp = client.assemble_quote(quote).await?.ok_or_else(|| eyre::eyre!("No bundle found"))?;

    execute_bundle(wallet, resp.match_bundle.settlement_tx).await
}
