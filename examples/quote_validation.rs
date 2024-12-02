use std::{str::FromStr, sync::Arc};

use ethers::middleware::Middleware;
use ethers::prelude::*;
use renegade_sdk::{
    types::{ApiExternalQuote, AtomicMatchApiBundle, ExternalOrder, OrderSide},
    ExternalMatchClient, ExternalOrderBuilder,
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
    let bundle = match client.assemble_quote(signed_quote).await? {
        Some(bundle) => bundle,
        None => eyre::bail!("No bundle found"),
    };
    execute_bundle(wallet, bundle).await
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
