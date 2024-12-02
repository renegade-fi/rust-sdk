# rust-sdk
A rust SDK for building Renegade clients.

## Basic Use

Todo...

# External (Atomic) Matches

In addition to the standard darkpool flow -- deposit, place order, receive a match, then withdraw -- Renegade also supports *external* matches. An external match is a match between an internal party -- with state committed into the darkpool -- and an external party, with no state in the darkpool. Importantly, external matches are settled atomically; that is, the deposit, place order, match, withdraw flow is emulated in a _single transaction_ for the external party.

An external match is generated and submitted on-chain by a client (see `ExternalMatchClient`). The client submits an `ExternalOrder` to the relayer, and the relayer will attempt to match it against all consenting internal orders. If a match is found, the relayer will respond to the client with a bundle containing:
- The match itself, specifying the amount and mint (ERC20 address) of the tokens bought and sold
- An EVM transaction that the external party may submit in order to settle the match with the darkpool

The client should then submit this match to the darkpool.

Upon receiving an external match, the darkpool contract will update the encrypted state of the internal party, and fulfill obligations to the external party directly through ERC20 transfers. As such, the external party must approve the token they _sell_ before the external match can be settled.

There are two ways to request an external match:

## Method 1: Quote + Assemble
This method is the recommended method as it has more permissive rate limits.
The code breaks down into two steps:
1. Fetch a quote for the order
2. If the quote is acceptable, assemble the transaction to submit on-chain

### Example
A full example can be found in [`examples/external_match.rs`](examples/external_match.rs).

This can be run with `cargo run --example external_match`.

<details>
<summary>Rust Code</summary>

```rust

// ... See `examples/external_match.rs` for full example ... //

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
    wallet: &OurMiddleware,
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
    let bundle = match client.assemble_quote(quote).await? {
        Some(bundle) => bundle,
        None => eyre::bail!("No bundle found"),
    };
    execute_bundle(wallet, bundle).await
}

/// Execute a bundle directly
async fn execute_bundle(
    wallet: &OurMiddleware,
    bundle: AtomicMatchApiBundle,
) -> Result<(), eyre::Error> {
    println!("Submitting bundle...\n");
    let tx = bundle.settlement_tx.clone();
    let receipt: PendingTransaction<_> = wallet.send_transaction(tx, None).await.unwrap();

    println!("Successfully submitted transaction: {:#x}", receipt.tx_hash());
    Ok(())
}
```
</details>

## Method 2: Direct Match
**Note:** It is recommended to use Method 1; this method is subject to strict rate limits.

Using this method, clients may directly request a match bundle from the relayer, without first requesting a quote.

### Example

<details>
<summary>Rust Code</summary>

```rust
use anyhow::{anyhow, Result};
use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
};
use renegade_sdk::{
    types::{AtomicMatchApiBundle, HmacKey, OrderSide},
    ExternalMatchClient, ExternalOrderBuilder,
};
use std::env;
use std::sync::Arc;

/// The quote mint for the atomic match
const QUOTE_MINT: &str = "0xdf8d259c04020562717557f2b5a3cf28e92707d1"; // USDC on Arbitrum Sepolia
/// The base mint for the atomic match
const BASE_MINT: &str = "0xc3414a7ef14aaaa9c4522dfc00a4e66e74e9c25a"; // wETH on Arbitrum Sepolia
/// The RPC URL for the Arbitrum Sepolia network
const ARBITRUM_SEPOLIA_RPC: &str = "..."; // replace with your RPC URL

#[tokio::main]
async fn main() -> Result<()> {
    // ... Token approvals before submitting settlement transaction ... //

    // Build the client and an order
    let api_key = env::var("API_KEY").expect("API_KEY must be set");
    let api_secret = env::var("API_SECRET").expect("API_SECRET must be set");
    let client = ExternalMatchClient::new_sepolia_client(&api_key, &api_secret)?;
    let order = ExternalOrderBuilder::new()
        .quote_mint(QUOTE_MINT)
        .base_mint(BASE_MINT)
        .side(OrderSide::Sell)
        .amount(100000000000000000) // 10^17 or 0.1 wETH
        .build()
        .unwrap();

    // Request and submit an external match
    let bundle = client.request_external_match(order).await.unwrap();
    match bundle {
        Some(bundle) => submit_settlement_transaction(bundle).await?,
        None => println!("No external match"),
    }

    Ok(())
}

/// Submit the settlement transaction
async fn submit_settlement_transaction(bundle: AtomicMatchApiBundle) -> Result<()> {
    let ethers_client = create_ethers_client().await?;
    let receipt = ethers_client
        .send_transaction(bundle.settlement_tx, None)
        .await?
        .await?
        .expect("no transaction receipt");

    println!(
        "Settlement transaction submitted. Hash: {:?}",
        receipt.transaction_hash
    );
    Ok(())
}

/// Create an Ethers client for Arbitrum Sepolia
async fn create_ethers_client() -> Result<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>> {
    let provider = Provider::<Http>::try_from(ARBITRUM_SEPOLIA_RPC)?;
    let chain_id = provider.get_chainid().await?.as_u64();

    let private_key = env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");
    let wallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);

    Ok(Arc::new(SignerMiddleware::new(provider, wallet)))
}
```

</details>

## Gas Estimation

You can also request that the relayer estimate gas for the settlement transaction by using `request_external_match_with_options` as below:
```rust
async fn request_match() -> Result<> {
    // ... Build client and order ... // 
    let options = ExternalMatchOptions::new().with_gas_estimation(true);
    let bundle = client
        .request_external_match_with_options(order, options)
        .await?;
    println!("Gas estimate: {:?}", bundle.settlement_tx.gas());

    // ... Submit Settlement Transaction ... //
}
```

## Bundle Details
The *quote* returned by the relayer for an external match has the following structure:
- `order`: The original external order
- `match_result`: The result of the match, including:
- `fees`: The fees for the match
    - `relayer_fee`: The fee paid to the relayer
    - `protocol_fee`: The fee paid to the protocol
- `receive`: The asset transfer the external party will receive, *after fees are deducted*.
    - `mint`: The token address
    - `amount`: The amount to receive
- `send`: The asset transfer the external party needs to send. No fees are charged on the send transfer. (same fields as `receive`)
- `price`: The price used for the match
- `timestamp`: The timestamp of the quote

When assembled into a bundle (returned from `assemble_quote` or `request_external_match`), the structure is as follows:
- `match_result`: The final match result
- `fees`: The fees to be paid
- `receive`: The asset transfer the external party will receive
- `send`: The asset transfer the external party needs to send
- `settlement_tx`: The transaction to submit on-chain
    - `tx_type`: The transaction type
    - `to`: The contract address
    - `data`: The calldata
    - `value`: The ETH value to send

See example [`quote_validation.rs`](examples/quote_validation.rs) for an example of using these fields to validate a quote before submitting it.

This can be run with `cargo run --example quote_validation`.