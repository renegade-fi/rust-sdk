use renegade_sdk::example_utils::{Wallet, build_renegade_client, execute_bundle, get_signer};
use renegade_sdk::{
    ExternalMatchClient, ExternalOrderBuilder,
    types::{ExternalOrder, OrderSide},
};

/// Testnet cbBTC
const BASE_MINT: &str = "0xb51a558c8E55DE1EE5391BDFe2aFA49968FC3B25";
/// Testnet USDC
const QUOTE_MINT: &str = "0xD9961Bb4Cb27192f8dAd20a662be081f546b0E74";

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Get wallet from private key
    let signer = get_signer().await?;

    // Get the external match client
    let client = build_renegade_client(true /* use_base */)?;
    let order = ExternalOrderBuilder::new()
        .base_mint(BASE_MINT)
        .quote_mint(QUOTE_MINT)
        .quote_amount(30_000_000) // $30 USDC
        .min_fill_size(30_000_000) // $30 USDC
        .side(OrderSide::Sell)
        .build()
        .unwrap();

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
