use rand::Rng;
use renegade_sdk::api_types::MalleableExternalMatchResponseWithConnector;
use renegade_sdk::example_utils::{build_renegade_client, execute_bundle, get_signer, Wallet};
use renegade_sdk::{
    types::{ExternalOrder, OrderSide},
    AssembleQuoteOptions, ExternalMatchClient, ExternalOrderBuilder,
};

/// Testnet wETH
const BASE_MINT: &str = "0x31a5552AF53C35097Fdb20FFf294c56dc66FA04c";
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
        .min_fill_size(1_000_000) // $1 USDC
        .side(OrderSide::Sell)
        .build()
        .unwrap();

    fetch_quote_and_execute_connector_malleable(&client, order, &signer).await?;
    Ok(())
}

/// Fetch a quote from the external api and execute a malleable match using the
/// connector contract
///
/// The connector contract ABI allows specifying only the input amount in the
/// calldata, rather than both base and quote amounts. This simplifies the
/// calldata modification process - when selling the base token, only the base
/// amount is set in calldata; when buying the base token, only the quote amount
/// is set.
async fn fetch_quote_and_execute_connector_malleable(
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

    // Assemble the quote into a malleable bundle using the connector contract
    println!("Assembling malleable quote with connector contract...");
    let options = AssembleQuoteOptions::new().use_connector_contract();
    let mut bundle: MalleableExternalMatchResponseWithConnector =
        match client.assemble_malleable_quote_with_options(quote, options).await? {
            Some(resp) => resp,
            None => eyre::bail!("No malleable bundle found"),
        };

    // With the connector contract, we can use `set_input_amount` which
    // automatically chooses whether to set base or quote based on trade direction
    // Alternatively, you can explicitly set base or quote amounts - see
    // `set_random_base_amount` and `set_random_quote_amount` below
    set_random_input_amount(&mut bundle);

    // Execute the bundle
    println!("Executing malleable match bundle...");
    let tx = bundle.settlement_tx();
    execute_bundle(wallet, tx).await
}

/// Set a random input amount on the bundle using the connector-specific method
///
/// The `set_input_amount` method automatically determines whether to set the
/// base amount or quote amount based on which token is being sold. This is
/// convenient since the connector contract only requires the input amount in
/// calldata.
fn set_random_input_amount(bundle: &mut MalleableExternalMatchResponseWithConnector) {
    println!("\nBundle info:");
    let (min_base, max_base) = bundle.base_bounds();
    let (min_quote, max_quote) = bundle.quote_bounds();
    println!("\tBase bounds: {min_base} - {max_base}");
    println!("\tQuote bounds: {min_quote} - {max_quote}");

    let mut rng = rand::thread_rng();

    // Determine which amount to use as input based on trade direction
    // The connector contract uses the input amount (base when selling, quote when
    // buying)
    let sells_base = bundle.sells_base_token();
    let (min_input, max_input) =
        if sells_base { (min_base, max_base) } else { (min_quote, max_quote) };

    // Pick an actual input amount to swap with
    let swapped_input_amt = rng.gen_range(min_input..=max_input);

    // Setting the input amount automatically handles base vs quote based on
    // which token is being sold
    let _recv = bundle.set_input_amount(swapped_input_amt).unwrap();
    let send = bundle.send_amount();
    let recv = bundle.receive_amount();
    println!("\tSwapped input amount: {swapped_input_amt}");
    println!("\tSend amount: {send}");
    println!("\tReceived amount: {recv}\n\n");
}
