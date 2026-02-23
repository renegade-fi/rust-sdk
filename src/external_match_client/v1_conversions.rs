//! Conversion functions between v1 and v2 external match types

use std::collections::HashSet;

use alloy::{primitives::U256, sol_types::SolValue};

use crate::api_types::{
    ApiExternalAssetTransfer, ApiSignedQuoteV2, ApiTimestampedPrice, ExternalMatchResponseV2,
    ExternalOrderV2, FeeTake, OrderSide, SignedExternalQuoteV2,
    markets::MarketDepth,
    v1_types::{
        ApiExternalMatchResult, ApiExternalQuote, AtomicMatchApiBundle, ExternalMatchResponse,
        ExternalOrder, FeeRates, GetDepthByMintResponse, GetDepthForAllPairsResponse,
        GetSupportedTokensResponse, GetTokenPricesResponse, PriceAndDepth, SignedExternalQuote,
        SignedGasSponsorshipInfo, TokenPrice,
    },
};

use super::{
    ExternalMatchClientError,
    api_types::{GetMarketDepthByMintResponse, GetMarketDepthsResponse, GetMarketsResponse},
};

/// The offset of the input amount in the calldata, after the 4-byte function
/// selector
const INPUT_AMOUNT_OFFSET: usize = 4;
/// The length of an amount in calldata (32 bytes for a `uint256`)
const AMOUNT_CALLDATA_LENGTH: usize = 32;

// -------------------------
// | ExternalOrder v1 → v2 |
// -------------------------

/// Convert a v1 `ExternalOrder` to a v2 `ExternalOrderV2`
pub(crate) fn v1_order_to_v2(order: &ExternalOrder) -> ExternalOrderV2 {
    match order.side {
        OrderSide::Buy => {
            // Buy: input=quote, output=base
            let (input_amount, output_amount, use_exact_output_amount) = if order.quote_amount != 0
            {
                (order.quote_amount, 0, false)
            } else if order.base_amount != 0 {
                (0, order.base_amount, false)
            } else if order.exact_base_output != 0 {
                (0, order.exact_base_output, true)
            } else {
                (order.exact_quote_output, 0, true)
            };

            ExternalOrderV2 {
                input_mint: order.quote_mint.clone(),
                output_mint: order.base_mint.clone(),
                input_amount,
                output_amount,
                use_exact_output_amount,
                min_fill_size: order.min_fill_size,
            }
        },
        OrderSide::Sell => {
            // Sell: input=base, output=quote
            let (input_amount, output_amount, use_exact_output_amount) = if order.base_amount != 0 {
                (order.base_amount, 0, false)
            } else if order.quote_amount != 0 {
                (0, order.quote_amount, false)
            } else if order.exact_quote_output != 0 {
                (0, order.exact_quote_output, true)
            } else {
                (order.exact_base_output, 0, true)
            };

            ExternalOrderV2 {
                input_mint: order.base_mint.clone(),
                output_mint: order.quote_mint.clone(),
                input_amount,
                output_amount,
                use_exact_output_amount,
                min_fill_size: order.min_fill_size,
            }
        },
    }
}

// ----------------------------------------------
// | ExternalMatchResponseV2 → v1 Non-Malleable |
// ----------------------------------------------

/// Decode the input amount from the settlement transaction calldata
fn decode_input_amount_from_calldata(
    resp: &ExternalMatchResponseV2,
) -> Result<u128, ExternalMatchClientError> {
    let data = resp.match_bundle.settlement_tx.input.input();
    let data = data.unwrap_or_default();
    let end = INPUT_AMOUNT_OFFSET + AMOUNT_CALLDATA_LENGTH;

    if data.len() < end {
        return Err(ExternalMatchClientError::deserialize("invalid calldata"));
    }

    let input_slice = &data[INPUT_AMOUNT_OFFSET..end];
    let input_u256 =
        U256::abi_decode(input_slice).map_err(ExternalMatchClientError::deserialize)?;

    Ok(input_u256.to::<u128>())
}

/// Convert a v2 `ExternalMatchResponseV2` to a v1 non-malleable
/// `ExternalMatchResponse`
pub(crate) fn v2_response_to_v1_non_malleable(
    v2_match_resp: ExternalMatchResponseV2,
    direction: OrderSide,
) -> Result<ExternalMatchResponse, ExternalMatchClientError> {
    let match_result = &v2_match_resp.match_bundle.match_result;
    // The v2 price_fp is output/input. We use it to derive the output amount
    // from the input amount set in the calldata.
    // This price will have been updated to account for gas sponsorship, if the
    // match was sponsored. Thus, the output amount calculated with this price
    // also accounts for sponsorship.
    let price_fp = &match_result.price_fp;
    let input_amount = decode_input_amount_from_calldata(&v2_match_resp)?;
    let output_amount = price_fp.floor_mul_int(input_amount);

    // Map v2 input/output to v1 base/quote based on direction:
    //   Buy:  input=quote, output=base
    //   Sell: input=base, output=quote
    let (quote_mint, base_mint, quote_amount, base_amount) = match direction {
        OrderSide::Buy => (
            match_result.input_mint.clone(),
            match_result.output_mint.clone(),
            input_amount,
            output_amount,
        ),
        OrderSide::Sell => (
            match_result.output_mint.clone(),
            match_result.input_mint.clone(),
            output_amount,
            input_amount,
        ),
    };

    let v1_match_result = ApiExternalMatchResult {
        quote_mint: quote_mint.clone(),
        base_mint: base_mint.clone(),
        quote_amount,
        base_amount,
        direction: direction.clone(),
    };

    // Compute fees from fee_rates and output amount
    let total_fee_rate = v2_match_resp.match_bundle.fee_rates.total();
    let total_fee = total_fee_rate.floor_mul_int(output_amount);
    let relayer_fee =
        v2_match_resp.match_bundle.fee_rates.relayer_fee_rate.floor_mul_int(output_amount);
    let protocol_fee = total_fee - relayer_fee;
    let fees = FeeTake { relayer_fee, protocol_fee };

    // Compute receive/send from direction.
    // The receive token is the output token; fees are subtracted from receive.
    let (receive, send) = match direction {
        OrderSide::Buy => {
            let recv_amount = base_amount - total_fee;
            (
                ApiExternalAssetTransfer { mint: base_mint, amount: recv_amount },
                ApiExternalAssetTransfer { mint: quote_mint, amount: quote_amount },
            )
        },
        OrderSide::Sell => {
            let recv_amount = quote_amount - total_fee;
            (
                ApiExternalAssetTransfer { mint: quote_mint, amount: recv_amount },
                ApiExternalAssetTransfer { mint: base_mint, amount: base_amount },
            )
        },
    };

    let gas_sponsored = v2_match_resp.gas_sponsorship_info.is_some();
    let bundle = AtomicMatchApiBundle {
        match_result: v1_match_result,
        fees,
        receive,
        send,
        settlement_tx: v2_match_resp.match_bundle.settlement_tx,
        deadline: v2_match_resp.match_bundle.deadline,
    };

    Ok(ExternalMatchResponse {
        match_bundle: bundle,
        gas_sponsored,
        gas_sponsorship_info: v2_match_resp.gas_sponsorship_info,
    })
}

// --------------------------------------------------
// | SignedExternalQuoteV2 → v1 SignedExternalQuote |
// --------------------------------------------------

/// Invert a price string (compute 1/price) for converting between
/// output/input and quote/base conventions on Buy orders
fn invert_price_string(price_str: &str) -> Result<String, ExternalMatchClientError> {
    let price: f64 = price_str.parse().map_err(ExternalMatchClientError::deserialize)?;
    if price == 0.0 {
        return Ok("0".to_string());
    }
    let inverted = 1.0 / price;
    Ok(inverted.to_string())
}

/// Convert a v2 `SignedExternalQuoteV2` to a v1 `SignedExternalQuote`
pub(crate) fn v2_quote_to_v1(
    v2_signed_quote: SignedExternalQuoteV2,
    original_order: &ExternalOrder,
) -> Result<SignedExternalQuote, ExternalMatchClientError> {
    let v2_quote = &v2_signed_quote.quote;

    // Determine direction from the original v1 order
    let direction = &original_order.side;

    let (quote_mint, base_mint, quote_amount, base_amount) = match direction {
        OrderSide::Buy => (
            v2_quote.match_result.input_mint.clone(),
            v2_quote.match_result.output_mint.clone(),
            v2_quote.match_result.input_amount,
            v2_quote.match_result.output_amount,
        ),
        OrderSide::Sell => (
            v2_quote.match_result.output_mint.clone(),
            v2_quote.match_result.input_mint.clone(),
            v2_quote.match_result.output_amount,
            v2_quote.match_result.input_amount,
        ),
    };

    let v1_match_result = ApiExternalMatchResult {
        quote_mint: quote_mint.clone(),
        base_mint: base_mint.clone(),
        quote_amount,
        base_amount,
        direction: direction.clone(),
    };

    // Convert the price from v2's output/input to v1's quote/base.
    // For Sell orders: output=quote, input=base → output/input = quote/base ✓
    // For Buy orders: output=base, input=quote → output/input = base/quote,
    //   so we invert to get quote/base.
    let v1_price = match direction {
        OrderSide::Buy => ApiTimestampedPrice {
            price: invert_price_string(&v2_quote.price.price)?,
            timestamp: v2_quote.price.timestamp,
        },
        OrderSide::Sell => v2_quote.price.clone(),
    };

    let v1_quote = ApiExternalQuote {
        order: original_order.clone(),
        match_result: v1_match_result,
        fees: v2_quote.fees,
        send: v2_quote.send.clone(),
        receive: v2_quote.receive.clone(),
        price: v1_price,
        timestamp: v2_quote.timestamp,
    };

    let gas_sponsorship_info = v2_signed_quote
        .gas_sponsorship_info
        .as_ref()
        .map(|info| SignedGasSponsorshipInfo { gas_sponsorship_info: info.clone() });

    // Store the original v2 ApiSignedQuote for round-tripping
    let inner_v2 = ApiSignedQuoteV2 {
        quote: v2_signed_quote.quote,
        signature: v2_signed_quote.signature.clone(),
        deadline: v2_signed_quote.deadline,
    };

    Ok(SignedExternalQuote {
        quote: v1_quote,
        signature: v2_signed_quote.signature,
        deadline: v2_signed_quote.deadline,
        gas_sponsorship_info,
        inner_v2_quote: inner_v2,
    })
}

// --------------------------------------------------
// | v1 SignedExternalQuote → v2 for round-tripping |
// --------------------------------------------------

/// Extract the v2 `SignedExternalQuoteV2` from a v1 `SignedExternalQuote`
/// for use in the assemble flow.
pub(crate) fn v1_quote_to_v2(v1: &SignedExternalQuote) -> SignedExternalQuoteV2 {
    let gas_info = v1.gas_sponsorship_info.as_ref().map(|s| s.gas_sponsorship_info.clone());
    SignedExternalQuoteV2 {
        quote: v1.inner_v2_quote.quote.clone(),
        signature: v1.inner_v2_quote.signature.clone(),
        deadline: v1.inner_v2_quote.deadline,
        gas_sponsorship_info: gas_info,
    }
}

// -------------------------------
// | Market response conversions |
// -------------------------------

/// Convert a `GetMarketsResponse` to a v1 `GetSupportedTokensResponse`
pub(crate) fn markets_to_supported_tokens(resp: &GetMarketsResponse) -> GetSupportedTokensResponse {
    let mut seen = HashSet::new();
    let mut tokens = Vec::new();
    for market in &resp.markets {
        if seen.insert(market.base.address.clone()) {
            tokens.push(market.base.clone());
        }
        if seen.insert(market.quote.address.clone()) {
            tokens.push(market.quote.clone());
        }
    }
    GetSupportedTokensResponse { tokens }
}

/// Convert a `GetMarketsResponse` to a v1 `GetTokenPricesResponse`
pub(crate) fn markets_to_token_prices(
    resp: &GetMarketsResponse,
) -> Result<GetTokenPricesResponse, ExternalMatchClientError> {
    let mut token_prices = Vec::new();
    for m in &resp.markets {
        let price: f64 = m.price.price.parse().map_err(ExternalMatchClientError::deserialize)?;
        token_prices.push(TokenPrice {
            base_token: m.base.address.clone(),
            quote_token: m.quote.address.clone(),
            price,
        });
    }
    Ok(GetTokenPricesResponse { token_prices })
}

/// Convert a `GetMarketDepthByMintResponse` to a v1 `GetDepthByMintResponse`
pub(crate) fn market_depth_to_v1(
    resp: &GetMarketDepthByMintResponse,
) -> Result<GetDepthByMintResponse, ExternalMatchClientError> {
    Ok(GetDepthByMintResponse { depth: market_depth_to_price_and_depth(&resp.market_depth)? })
}

/// Convert a `GetMarketDepthsResponse` to a v1 `GetDepthForAllPairsResponse`
pub(crate) fn market_depths_to_v1(
    resp: &GetMarketDepthsResponse,
) -> Result<GetDepthForAllPairsResponse, ExternalMatchClientError> {
    let pairs = resp
        .market_depths
        .iter()
        .map(market_depth_to_price_and_depth)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(GetDepthForAllPairsResponse { pairs })
}

/// Convert a single `MarketDepth` to a v1 `PriceAndDepth`
fn market_depth_to_price_and_depth(
    depth: &MarketDepth,
) -> Result<PriceAndDepth, ExternalMatchClientError> {
    let price: f64 =
        depth.market.price.price.parse().map_err(ExternalMatchClientError::deserialize)?;
    Ok(PriceAndDepth {
        address: depth.market.base.address.clone(),
        price,
        timestamp: depth.market.price.timestamp,
        buy: depth.buy.clone(),
        sell: depth.sell.clone(),
        fee_rates: FeeRates {
            relayer_fee_rate: depth.market.external_match_fee_rates.relayer_fee_rate.to_f64(),
            protocol_fee_rate: depth.market.external_match_fee_rates.protocol_fee_rate.to_f64(),
        },
    })
}

// --------------------------------
// | AssembleQuoteOptions v1 → v2 |
// --------------------------------

/// Convert v1 `AssembleQuoteOptions` to v2 `AssembleQuoteOptionsV2`
pub(crate) fn v1_assemble_options_to_v2(
    opts: &super::options::AssembleQuoteOptions,
) -> crate::AssembleQuoteOptionsV2 {
    crate::AssembleQuoteOptionsV2 {
        do_gas_estimation: opts.do_gas_estimation,
        receiver_address: opts.receiver_address.clone(),
        updated_order: opts.updated_order.as_ref().map(v1_order_to_v2),
    }
}
