//! v1-compatible type definitions for backwards compatibility
//!
//! These types mirror the v1 SDK's API surface, using base/quote/side
//! terminology instead of v2's input/output terminology.

use alloy_rpc_types_eth::TransactionRequest;
use serde::{Deserialize, Serialize};

use super::{
    Amount, ApiExternalAssetTransfer, ApiSignedQuoteV2, ApiTimestampedPrice, FeeTake,
    GasSponsorshipInfo, OrderSide, markets::DepthSide, token::ApiToken,
};

// -----------------
// | Order Types   |
// -----------------

/// An external order (v1 format with base/quote/side)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalOrder {
    /// The mint (erc20 address) of the quote token
    pub quote_mint: String,
    /// The mint (erc20 address) of the base token
    pub base_mint: String,
    /// The side of the market this order is on
    pub side: OrderSide,
    /// The base amount of the order
    #[serde(default)]
    pub base_amount: Amount,
    /// The quote amount of the order
    #[serde(default)]
    pub quote_amount: Amount,
    /// The exact base amount to output from the order
    #[serde(default)]
    pub exact_base_output: Amount,
    /// The exact quote amount to output from the order
    #[serde(default)]
    pub exact_quote_output: Amount,
    /// The minimum fill size for the order
    #[serde(default)]
    pub min_fill_size: Amount,
}

// --------------------
// | Match Results    |
// --------------------

/// An API server external match result (v1 format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiExternalMatchResult {
    /// The mint of the quote token in the matched asset pair
    pub quote_mint: String,
    /// The mint of the base token in the matched asset pair
    pub base_mint: String,
    /// The amount of the quote token exchanged by the match
    pub quote_amount: Amount,
    /// The amount of the base token exchanged by the match
    pub base_amount: Amount,
    /// The direction of the match
    pub direction: OrderSide,
}

// --------------------
// | Bundle Types     |
// --------------------

/// An atomic match settlement bundle (v1 non-malleable format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AtomicMatchApiBundle {
    /// The match result
    pub match_result: ApiExternalMatchResult,
    /// The fees owed by the external party
    pub fees: FeeTake,
    /// The transfer received by the external party, net of fees
    pub receive: ApiExternalAssetTransfer,
    /// The transfer sent by the external party
    pub send: ApiExternalAssetTransfer,
    /// The transaction which settles the match on-chain
    pub settlement_tx: TransactionRequest,
    /// The deadline of the bundle, in milliseconds since the epoch
    pub deadline: u64,
}

// --------------------
// | Response Types   |
// --------------------

/// The response type for a non-malleable external match (v1 format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalMatchResponse {
    /// The raw response from the relayer
    pub match_bundle: AtomicMatchApiBundle,
    /// Whether the match has received gas sponsorship
    #[serde(rename = "is_sponsored", default)]
    pub gas_sponsored: bool,
    /// The gas sponsorship info, if the match was sponsored
    pub gas_sponsorship_info: Option<GasSponsorshipInfo>,
}

// --------------------
// | Quote Types      |
// --------------------

/// A quote for an external order (v1 format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiExternalQuote {
    /// The external order
    pub order: ExternalOrder,
    /// The match result
    pub match_result: ApiExternalMatchResult,
    /// The estimated fees for the match
    pub fees: FeeTake,
    /// The amount sent by the external party
    pub send: ApiExternalAssetTransfer,
    /// The amount received by the external party, net of fees
    pub receive: ApiExternalAssetTransfer,
    /// The price of the match
    pub price: ApiTimestampedPrice,
    /// The timestamp of the quote
    pub timestamp: u64,
}

/// A signed quote directly returned by the auth server (v1 format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiSignedQuote {
    /// The quote
    pub quote: ApiExternalQuote,
    /// The signature
    pub signature: String,
    /// The deadline of the quote, in milliseconds since the epoch
    pub deadline: u64,
}

/// A signed quote for an external order, including gas sponsorship info (v1
/// format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedExternalQuote {
    /// The quote
    pub quote: ApiExternalQuote,
    /// The signature
    pub signature: String,
    /// The deadline of the quote, in milliseconds since the epoch
    pub deadline: u64,
    /// The signed gas sponsorship info, if sponsorship was requested
    pub gas_sponsorship_info: Option<SignedGasSponsorshipInfo>,
    /// The original v2 signed quote, stored for round-tripping through the
    /// assemble flow without breaking the signature
    #[serde(skip)]
    pub(crate) inner_v2_quote: ApiSignedQuoteV2,
}

impl SignedExternalQuote {
    /// Get the match result from the quote
    pub fn match_result(&self) -> ApiExternalMatchResult {
        self.quote.match_result.clone()
    }

    /// Get the fees from the quote
    pub fn fees(&self) -> FeeTake {
        self.quote.fees
    }

    /// Get the receive amount from the quote
    pub fn receive_amount(&self) -> ApiExternalAssetTransfer {
        self.quote.receive.clone()
    }

    /// Get the send amount from the quote
    pub fn send_amount(&self) -> ApiExternalAssetTransfer {
        self.quote.send.clone()
    }
}

/// Signed metadata regarding gas sponsorship for a quote (v1 format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedGasSponsorshipInfo {
    /// The gas sponsorship info
    pub gas_sponsorship_info: GasSponsorshipInfo,
}

// -------------------------
// | Market / Token Types  |
// -------------------------

/// The response type to fetch the supported token list (v1 format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetSupportedTokensResponse {
    /// The supported tokens
    pub tokens: Vec<ApiToken>,
}

/// The response type to fetch the token prices (v1 format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetTokenPricesResponse {
    /// The token prices
    pub token_prices: Vec<TokenPrice>,
}

/// Price information for a token (v1 format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenPrice {
    /// The mint (ERC20 address) of the base token
    pub base_token: String,
    /// The mint (ERC20 address) of the quote token
    pub quote_token: String,
    /// The price data for this token
    pub price: f64,
}

/// Response for the GET /order_book/depth/:mint route (v1 format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetDepthByMintResponse {
    /// The liquidity depth for the given mint
    #[serde(flatten)]
    pub depth: PriceAndDepth,
}

/// Response for the GET /order_book/depth route (v1 format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetDepthForAllPairsResponse {
    /// The liquidity depth for all supported pairs
    pub pairs: Vec<PriceAndDepth>,
}

/// The fee rates for a given pair (v1 format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeeRates {
    /// The relayer fee rate
    pub relayer_fee_rate: f64,
    /// The protocol fee rate
    pub protocol_fee_rate: f64,
}

/// The liquidity depth for a given pair (v1 format)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PriceAndDepth {
    /// The token address
    pub address: String,
    /// The current price of the token in USD
    pub price: f64,
    /// The timestamp of the price
    pub timestamp: u64,
    /// The liquidity depth for the buy side
    pub buy: DepthSide,
    /// The liquidity depth for the sell side
    pub sell: DepthSide,
    /// The fee rates for the pair
    pub fee_rates: FeeRates,
}
