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

### Generating an External Match

Generating an external match breaks down into three steps:
1. Fetch a quote for the order.
2. If the quote is acceptable, assemble the quote into a **bundle**. Bundles contain a transaction that may be used to settle the trade on-chain.
3. Submit the settlement transaction on-chain.

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

### Rate Limits
The rate limits for external match endpoints are as follows: 
- **Quote**: 100 requests per minute
- **Assemble**: 5 _unsettled_ bundles per minute. That is, if an assembled bundle is submitted on-chain, the rate limiter will reset. 
If an assembled match is not settled on-chain, the rate limiter will remove one token from the per-minute allowance.