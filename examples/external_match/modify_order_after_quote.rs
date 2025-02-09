//! An example in which we tweak an order before assembling the quote

use std::{str::FromStr, sync::Arc};

use ethers::middleware::Middleware;
use ethers::prelude::*;
use renegade_sdk::{
    types::{AtomicMatchApiBundle, ExternalOrder, OrderSide},
    AssembleQuoteOptions, ExternalMatchClient, ExternalOrderBuilder,
};

/// The RPC URL to use
const RPC_URL: &str = env!("RPC_URL");

/// Testnet wETH
const BASE_MINT: &str = "0xc3414a7ef14aaaa9c4522dfc00a4e66e74e9c25a";
/// Testnet USDC
const QUOTE_MINT: &str = "0xdf8d259c04020562717557f2b5a3cf28e92707d1";

/// The middleware type
type Wallet = Arc<SignerMiddleware<Provider<Http>, LocalWallet>>;

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Get wallet from private key
    let signer = get_signer().await?;

    // Get the external match client
    let api_key = std::env::var("EXTERNAL_MATCH_KEY").unwrap();
    let api_secret = std::env::var("EXTERNAL_MATCH_SECRET").unwrap();
    let client = ExternalMatchClient::new_sepolia_client(&api_key, &api_secret).unwrap();

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
    let res = client.request_quote(order.clone()).await?;
    let quote = match res {
        Some(quote) => quote,
        None => eyre::bail!("No quote found"),
    };

    // We only want to sell $29 of WETH, not $30
    let mut updated_order = order.clone();
    updated_order.quote_amount = 29_000_000;
    updated_order.min_fill_size = 29_000_000;

    // Assemble the quote into a bundle
    println!("Assembling quote...");
    let options = AssembleQuoteOptions::default().with_updated_order(updated_order);
    let resp = match client.assemble_quote_with_options(quote, options).await? {
        Some(resp) => resp,
        None => eyre::bail!("No bundle found"),
    };
    execute_bundle(wallet, resp.match_bundle).await
}

/// Execute a bundle directly
async fn execute_bundle(wallet: &Wallet, bundle: AtomicMatchApiBundle) -> Result<(), eyre::Error> {
    println!("Submitting bundle...\n");
    let tx = bundle.settlement_tx.clone();
    let receipt: PendingTransaction<_> = wallet.send_transaction(tx, None).await.unwrap();

    println!("Successfully submitted transaction: {:#x}", receipt.tx_hash());
    Ok(())
}

// -----------
// | Helpers |
// -----------

/// Get a wallet from a private key environment variable
async fn get_signer() -> Result<Wallet, eyre::Error> {
    let provider = Provider::<Http>::try_from(RPC_URL).unwrap();
    let chain_id = provider.get_chainid().await.unwrap().as_u64();
    let pkey = std::env::var("PKEY").unwrap();
    let wallet = LocalWallet::from_str(&pkey).unwrap().with_chain_id(chain_id);
    let middleware = Arc::new(SignerMiddleware::new(provider, wallet));

    Ok(middleware)
}
