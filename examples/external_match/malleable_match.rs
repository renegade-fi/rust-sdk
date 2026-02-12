use rand::Rng;
use renegade_sdk::api_types::ExternalMatchResponseV2;
use renegade_sdk::example_utils::{Wallet, build_renegade_client, execute_bundle, get_signer};
use renegade_sdk::{ExternalMatchClient, ExternalOrderBuilderV2};

/// Testnet wETH
const WETH_MINT: &str = "0xc3414a7ef14aaaa9c4522dfc00a4e66e74e9c25a";
/// Testnet USDC
const USDC_MINT: &str = "0xdf8d259c04020562717557f2b5a3cf28e92707d1";

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Get wallet from private key
    let signer = get_signer().await?;

    // Get the external match client
    let client = build_renegade_client(false /* use_base */)?;
    let order = ExternalOrderBuilderV2::new()
        .input_mint(WETH_MINT)
        .output_mint(USDC_MINT)
        .input_amount(100000000000000) // $30 USDC
        .min_fill_size(10000000000000) // $1 USDC
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
    order: renegade_sdk::api_types::ExternalOrderV2,
    wallet: &Wallet,
) -> Result<(), eyre::Error> {
    // Fetch a quote from the relayer
    println!("Fetching quote...");
    let res = client.request_quote_v2(order).await?;
    let quote = match res {
        Some(quote) => quote,
        None => eyre::bail!("No quote found"),
    };

    // Assemble the quote into a malleable bundle
    println!("Assembling quote...");
    let mut bundle: ExternalMatchResponseV2 = match client.assemble_quote_v2(quote).await? {
        Some(resp) => resp,
        None => eyre::bail!("No bundle found"),
    };

    // Set a random input amount on the bundle
    set_random_input_amount(&mut bundle);

    // Execute the bundle
    println!("Executing malleable match bundle...");
    let tx = bundle.settlement_tx();
    println!("tx.to: {:?}", tx.to);
    execute_bundle(wallet, tx).await
}

/// Set a random input amount on the bundle, and print the results
fn set_random_input_amount(bundle: &mut ExternalMatchResponseV2) {
    // Print bundle info
    println!("\nBundle info:");
    let (min_input, max_input) = bundle.input_bounds();
    println!("\tInput bounds: {min_input} - {max_input}");

    // Pick a random input amount and see the receive amount at that input
    let mut rng = rand::thread_rng();
    let dummy_input = rng.gen_range(min_input..=max_input);
    let recv = bundle.receive_amount_at_base(dummy_input);
    println!("\tHypothetical input amount: {dummy_input}");
    println!("\tHypothetical received amount: {recv}");

    // Pick an actual input amount to swap with
    let swapped_input = rng.gen_range(min_input..=max_input);

    // Setting the input amount will return the receive amount at the new input
    // You can also call send_amount and receive_amount to get the amounts at the
    // currently set input amount
    let _recv = bundle.set_input_amount(swapped_input).unwrap();
    let send = bundle.send_amount();
    let recv = bundle.receive_amount();
    println!("\tSwapped input amount: {swapped_input}");
    println!("\tSend amount: {send}");
    println!("\tReceived amount: {recv}\n\n");
}
