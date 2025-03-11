use renegade_sdk::{
    example_utils::{build_renegade_client, execute_bundle, get_signer, Wallet},
    types::{ApiExternalQuote, ExternalOrder, OrderSide},
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
        .base_amount(8000000000000000) // 0.8 WETH
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
    let signed_quote = match res {
        Some(quote) => quote,
        None => eyre::bail!("No quote found"),
    };

    // Validate the quote
    validate_quote(&signed_quote.quote).await?;

    // Assemble the quote into a bundle
    println!("Assembling quote...");
    let resp = match client.assemble_quote(signed_quote).await? {
        Some(resp) => resp,
        None => eyre::bail!("No bundle found"),
    };
    execute_bundle(wallet, resp.match_bundle).await
}

/// Validate a quote
async fn validate_quote(quote: &ApiExternalQuote) -> Result<(), eyre::Error> {
    const MIN_AMOUNT: u128 = 1000000000000000; // 0.001 WETH
    const MAX_FEE: u128 = 100000000000000; // 0.0001 WETH

    let total_fee = quote.fees.total();
    let recv_amount = quote.receive.amount;

    if recv_amount < MIN_AMOUNT {
        eyre::bail!("Received amount is less than the minimum amount");
    }

    if total_fee > MAX_FEE {
        eyre::bail!("Total fee is greater than the maximum fee");
    }

    Ok(())
}
