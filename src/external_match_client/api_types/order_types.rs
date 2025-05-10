//! Order types for the external match client

use alloy_rpc_types_eth::TransactionRequest;
use serde::{Deserialize, Serialize};

use super::FixedPoint;

// -------------
// | Api Types |
// -------------

/// A token in the the supported token list
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiToken {
    /// The token address
    pub address: String,
    /// The token symbol
    pub symbol: String,
}

/// A type alias for an amount used in the Renegade system
pub type Amount = u128;

/// The side of the market this order is on
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderSide {
    /// Buy side, requesting party buys the base token
    Buy,
    /// Sell side, requesting party sells the base token
    Sell,
}

/// An external order
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
    /// The minimum fill size for the order
    #[serde(default)]
    pub min_fill_size: Amount,
}

/// A fee take, representing the fee amounts paid to the relayer and protocol by
/// the external party when the match is settled
///
/// Fees are always paid in the receive token
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct FeeTake {
    /// The amount of fees paid to the relayer
    pub relayer_fee: Amount,
    /// The amount of fees paid to the protocol
    pub protocol_fee: Amount,
}

impl FeeTake {
    /// Get the total fee
    pub fn total(&self) -> Amount {
        self.relayer_fee + self.protocol_fee
    }
}

/// Fee rates for relayer and protocol
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeeTakeRate {
    /// The fee rate for the relayer
    pub relayer_fee_rate: FixedPoint,
    /// The fee rate for the protocol
    pub protocol_fee_rate: FixedPoint,
}

impl FeeTakeRate {
    /// Get the total fee rate
    pub fn total(&self) -> FixedPoint {
        &self.relayer_fee_rate + &self.protocol_fee_rate
    }
}

/// An API server bounded match result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiBoundedMatchResult {
    /// The mint of the quote token in the matched asset pair
    pub quote_mint: String,
    /// The mint of the base token in the matched asset pair
    pub base_mint: String,
    /// The price at which the match executes
    pub price_fp: FixedPoint,
    /// The minimum base amount of the match
    pub min_base_amount: Amount,
    /// The maximum base amount of the match
    pub max_base_amount: Amount,
    /// The direction of the match
    pub direction: OrderSide,
}

/// An atomic match settlement bundle using a malleable match result
///
/// A malleable match result is one in which the exact `base_amount` swapped
/// is not known at the time the proof is generated, and may be changed up until
/// it is submitted on-chain. Instead, a bounded match result gives a
/// `min_base_amount` and a `max_base_amount`, between which the `base_amount`
/// may take any value
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MalleableAtomicMatchApiBundle {
    /// The match result
    pub match_result: ApiBoundedMatchResult,
    /// The fees owed by the external party
    pub fee_rates: FeeTakeRate,
    /// The maximum amount that the external party will receive
    pub max_receive: ApiExternalAssetTransfer,
    /// The minimum amount that the external party will receive
    pub min_receive: ApiExternalAssetTransfer,
    /// The maximum amount that the external party will send
    pub max_send: ApiExternalAssetTransfer,
    /// The minimum amount that the external party will send
    pub min_send: ApiExternalAssetTransfer,
    /// The transaction which settles the match on-chain
    pub settlement_tx: TransactionRequest,
}

/// An atomic match settlement bundle, sent to the client so that they may
/// settle the match on-chain
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
}

/// An asset transfer from an external party
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiExternalAssetTransfer {
    /// The mint of the asset
    pub mint: String,
    /// The amount of the asset
    pub amount: Amount,
}

/// An API server external match result
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

/// A signed quote directly returned by the auth server
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiSignedQuote {
    /// The quote
    pub quote: ApiExternalQuote,
    /// The signature
    pub signature: String,
}

/// A signed quote for an external order, including gas sponsorship info, if any
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedExternalQuote {
    /// The quote
    pub quote: ApiExternalQuote,
    /// The signature
    pub signature: String,
    /// The signed gas sponsorship info, if sponsorship was requested
    pub gas_sponsorship_info: Option<SignedGasSponsorshipInfo>,
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

/// A quote for an external order
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

/// The price of a quote
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiTimestampedPrice {
    /// The price, serialized as a string to prevent floating point precision
    /// issues
    pub price: String,
    /// The timestamp, in milliseconds since the epoch
    pub timestamp: u64,
}

// -----------------------------
// | Gas Sponsorship API Types |
// -----------------------------

/// Signed metadata regarding gas sponsorship for a quote
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedGasSponsorshipInfo {
    /// The signed gas sponsorship info
    pub gas_sponsorship_info: GasSponsorshipInfo,
    /// The auth server's signature over the gas sponsorship info
    #[deprecated(since = "0.1.2", note = "Gas sponsorship info is no longer signed")]
    pub signature: String,
}

/// Metadata regarding gas sponsorship for a quote
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GasSponsorshipInfo {
    /// The amount to be refunded as a result of gas sponsorship.
    /// This amount is firm, it will not change when the quote is assembled.
    pub refund_amount: u128,
    /// Whether the refund is in terms of native ETH.
    pub refund_native_eth: bool,
    /// The address to which the refund will be sent, if set explicitly.
    pub refund_address: Option<String>,
}
