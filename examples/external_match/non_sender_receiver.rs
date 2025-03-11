//! An example requesting an external match where the message sender is not the
//! receiver

use renegade_sdk::{
    example_utils::{build_renegade_client, execute_bundle, get_signer, Wallet},
    types::{ExternalOrder, OrderSide},
    AssembleQuoteOptions, ExternalMatchClient, ExternalOrderBuilder,
};

/// Testnet wETH
const BASE_MINT: &str = "0xc3414a7ef14aaaa9c4522dfc00a4e66e74e9c25a";
/// Testnet USDC
const QUOTE_MINT: &str = "0xdf8d259c04020562717557f2b5a3cf28e92707d1";
/// The receiver address: the address that the darkpool will send funds to
const RECEIVER_ADDRESS: &str = "0xC5fE800A3D92112473e4E811296F194DA7b26BA7";

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
        .min_fill_size(30_000_000) // $30 USDC
        .side(OrderSide::Buy)
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
    let options = AssembleQuoteOptions::new().with_receiver_address(RECEIVER_ADDRESS.to_string());
    let resp = match client.assemble_quote_with_options(quote, options).await? {
        Some(resp) => resp,
        None => eyre::bail!("No bundle found"),
    };
    execute_bundle(wallet, resp.match_bundle).await
}
