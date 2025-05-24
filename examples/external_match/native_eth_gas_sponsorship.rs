//! An example requesting an external match with gas sponsorship

use renegade_sdk::{
    example_utils::{build_renegade_client, execute_bundle, get_signer, Wallet},
    types::{ExternalOrder, OrderSide},
    ExternalMatchClient, ExternalOrderBuilder, RequestQuoteOptions,
};

/// Testnet wETH
const BASE_MINT: &str = "0xc3414a7ef14aaaa9c4522dfc00a4e66e74e9c25a";
/// Testnet USDC
const QUOTE_MINT: &str = "0xdf8d259c04020562717557f2b5a3cf28e92707d1";
/// The gas refund address: the address that will receive the gas refund
const GAS_REFUND_ADDRESS: &str = "0x99D9133afE1B9eC1726C077cA2b79Dcbb5969707";

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Get wallet from private key
    let signer = get_signer().await?;

    // Get the external match client
    let client = build_renegade_client(false /* use_base */)?;
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
    println!("Fetching quote with native ETH gas sponsorship...");
    let options = RequestQuoteOptions::new()
        .with_refund_native_eth() // Request native ETH gas refund
        .with_gas_refund_address(GAS_REFUND_ADDRESS.to_string()); // Set the refund address

    let res = client.request_quote_with_options(order, options).await?;
    let quote = match res {
        Some(quote) => quote,
        None => eyre::bail!("No quote found"),
    };

    if quote.gas_sponsorship_info.is_none() {
        eyre::bail!("Quote was not sponsored");
    }

    if !quote.gas_sponsorship_info.as_ref().unwrap().gas_sponsorship_info.refund_native_eth {
        eyre::bail!("Quote was not sponsored in terms of native ETH");
    }

    // Assemble the quote into a bundle with gas sponsorship
    println!("Assembling quote...");
    let resp = match client.assemble_quote(quote).await? {
        Some(bundle) => bundle,
        None => eyre::bail!("No bundle found"),
    };

    if !resp.gas_sponsored {
        eyre::bail!("Bundle was not sponsored");
    }
    execute_bundle(wallet, resp.match_bundle.settlement_tx).await
}
