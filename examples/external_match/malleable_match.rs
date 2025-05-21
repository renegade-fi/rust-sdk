use rand::Rng;
use renegade_sdk::api_types::MalleableExternalMatchResponse;
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
        .side(OrderSide::Buy)
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
    let mut bundle = match client.assemble_malleable_quote(quote).await? {
        Some(resp) => resp,
        None => eyre::bail!("No malleable bundle found"),
    };

    // Set a base amount on the bundle
    // Alternatively, you can set a quote amount on the bundle - see
    // `set_random_quote_amount` below
    set_random_base_amount(&mut bundle);

    // Execute the bundle
    println!("Executing malleable match bundle...");
    let tx = bundle.settlement_tx();
    execute_bundle(wallet, tx).await
}

/// Set a random base amount on the bundle, and print the results
#[allow(dead_code)]
fn set_random_base_amount(bundle: &mut MalleableExternalMatchResponse) {
    // Print bundle info
    println!("\nBundle info:");
    let (min_base, max_base) = bundle.base_bounds();
    println!("\tBase bounds: {min_base} - {max_base}");

    // Pick a random base amount and see the send and receive amounts at that base
    // amount
    let mut rng = rand::thread_rng();
    let dummy_base_amount = rng.gen_range(min_base..=max_base);
    let send = bundle.send_amount_at_base(dummy_base_amount);
    let recv = bundle.receive_amount_at_base(dummy_base_amount);
    println!("\tHypothetical base amount: {dummy_base_amount}");
    println!("\tHypothetical send amount: {send}");
    println!("\tHypothetical received amount: {recv}");

    // Pick an actual base amount to swap with
    let swapped_base_amt = rng.gen_range(min_base..=max_base);

    // Setting the base amount will return the receive amount at the new base
    // You can also call send_amount and receive_amount to get the amounts at the
    // currently set base amount
    let _recv = bundle.set_base_amount(swapped_base_amt).unwrap();
    let send = bundle.send_amount();
    let recv = bundle.receive_amount();
    println!("\tSwapped base amount: {swapped_base_amt}");
    println!("\tSend amount: {send}");
    println!("\tReceived amount: {recv}\n\n");
}

/// Set a random quote amount on the bundle, and print the results
#[allow(dead_code)]
fn set_random_quote_amount(bundle: &mut MalleableExternalMatchResponse) {
    // Print bundle info
    println!("\nBundle info:");
    let (min_quote, max_quote) = bundle.quote_bounds();
    println!("\tQuote bounds: {min_quote} - {max_quote}");

    // Pick a random quote amount and see the send and receive amounts at that quote
    // amount
    let mut rng = rand::thread_rng();
    let dummy_quote_amount = rng.gen_range(min_quote..=max_quote);
    let send = bundle.send_amount_at_quote(dummy_quote_amount);
    let recv = bundle.receive_amount_at_quote(dummy_quote_amount);
    println!("\tHypothetical quote amount: {dummy_quote_amount}");
    println!("\tHypothetical send amount: {send}");
    println!("\tHypothetical received amount: {recv}");

    // Pick an actual quote amount to swap with
    let swapped_quote_amt = rng.gen_range(min_quote..=max_quote);

    // Setting the quote amount will return the receive amount at the new quote
    // You can also call send_amount and receive_amount to get the amounts at the
    // currently set quote amount
    let _recv = bundle.set_quote_amount(swapped_quote_amt).unwrap();
    let send = bundle.send_amount();
    let recv = bundle.receive_amount();
    println!("\tSwapped quote amount: {swapped_quote_amt}");
    println!("\tSend amount: {send}");
    println!("\tReceived amount: {recv}\n\n");
}
