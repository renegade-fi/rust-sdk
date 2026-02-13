//! Order types for the external match client

use alloy_rpc_types_eth::TransactionRequest;
use serde::{Deserialize, Serialize};

use super::{FixedPoint, serde_helpers::*};

// -------------
// | Api Types |
// -------------

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
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExternalOrderV2 {
    /// The mint (erc20 address) of the input token
    pub input_mint: String,
    /// The mint (erc20 address) of the output token
    pub output_mint: String,
    /// The input amount of the order
    #[serde(default)]
    #[serde(with = "amount_string_serde")]
    pub input_amount: Amount,
    /// The output amount of the order
    #[serde(default)]
    #[serde(with = "amount_string_serde")]
    pub output_amount: Amount,
    /// Whether to consider the output amount as an exact amount (net of fees)
    pub use_exact_output_amount: bool,
    /// The minimum fill size for the order
    #[serde(default)]
    #[serde(with = "amount_string_serde")]
    pub min_fill_size: Amount,
}

/// A fee take, representing the fee amounts paid to the relayer and protocol by
/// the external party when the match is settled
///
/// Fees are always paid in the receive token
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct FeeTake {
    /// The amount of fees paid to the relayer
    #[serde(with = "amount_string_serde")]
    pub relayer_fee: Amount,
    /// The amount of fees paid to the protocol
    #[serde(with = "amount_string_serde")]
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
pub struct ApiBoundedMatchResultV2 {
    /// The mint of the input token in the matched asset pair
    pub input_mint: String,
    /// The mint of the output token in the matched asset pair
    pub output_mint: String,
    /// The price of the match, in terms of output token per input token
    pub price_fp: FixedPoint,
    /// The minimum input amount of the match
    #[serde(with = "amount_string_serde")]
    pub min_input_amount: Amount,
    /// The maximum input amount of the match
    #[serde(with = "amount_string_serde")]
    pub max_input_amount: Amount,
}

/// An atomic match settlement bundle using a malleable match result
///
/// A malleable match result is one in which the exact `input_amount` swapped
/// is not known at the time the proof is generated, and may be changed up until
/// it is submitted on-chain. Instead, a bounded match result gives a
/// `min_input_amount` and a `max_input_amount`, between which the
/// `input_amount` may take any value
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MalleableAtomicMatchApiBundleV2 {
    /// The match result
    pub match_result: ApiBoundedMatchResultV2,
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
    /// The deadline of the bundle, in milliseconds since the epoch
    pub deadline: u64,
}

/// An asset transfer from an external party
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ApiExternalAssetTransfer {
    /// The mint of the asset
    pub mint: String,
    /// The amount of the asset
    #[serde(with = "amount_string_serde")]
    pub amount: Amount,
}

/// An API server external match result
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ApiExternalMatchResultV2 {
    /// The mint of the input token in the matched asset pair
    pub input_mint: String,
    /// The mint of the output token in the matched asset pair
    pub output_mint: String,
    /// The amount of the input token exchanged by the match
    #[serde(with = "amount_string_serde")]
    pub input_amount: Amount,
    /// The amount of the output token exchanged by the match
    #[serde(with = "amount_string_serde")]
    pub output_amount: Amount,
}

/// A signed quote directly returned by the auth server
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ApiSignedQuoteV2 {
    /// The quote
    pub quote: ApiExternalQuoteV2,
    /// The signature
    pub signature: String,
    /// The deadline of the quote, in milliseconds since the epoch
    pub deadline: u64,
}

impl From<SignedExternalQuoteV2> for ApiSignedQuoteV2 {
    fn from(signed_quote: SignedExternalQuoteV2) -> Self {
        ApiSignedQuoteV2 {
            quote: signed_quote.quote,
            signature: signed_quote.signature,
            deadline: signed_quote.deadline,
        }
    }
}

/// A signed quote for an external order, including gas sponsorship info, if any
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedExternalQuoteV2 {
    /// The quote
    pub quote: ApiExternalQuoteV2,
    /// The signature
    pub signature: String,
    /// The deadline of the quote, in milliseconds since the epoch
    pub deadline: u64,
    /// The signed gas sponsorship info, if sponsorship was requested
    pub gas_sponsorship_info: Option<GasSponsorshipInfo>,
}

impl SignedExternalQuoteV2 {
    /// Create a signed quote from an external quote
    pub fn from_api_quote(
        external_quote: ApiSignedQuoteV2,
        gas_sponsorship_info: Option<GasSponsorshipInfo>,
    ) -> Self {
        SignedExternalQuoteV2 {
            quote: external_quote.quote,
            signature: external_quote.signature,
            deadline: external_quote.deadline,
            gas_sponsorship_info,
        }
    }
    /// Get the match result from the quote
    pub fn match_result(&self) -> ApiExternalMatchResultV2 {
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
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ApiExternalQuoteV2 {
    /// The external order
    pub order: ExternalOrderV2,
    /// The match result
    pub match_result: ApiExternalMatchResultV2,
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
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
