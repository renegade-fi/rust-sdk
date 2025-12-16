//! Order types for the Renegade external match API

use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

use crate::v2::external_match_client::api::Amount;

use super::serde_helpers::*;

// ---------------
// | Order Types |
// ---------------

/// An external order
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalOrder {
    /// The mint (ERC20 address) of the input token
    pub input_mint: Address,
    /// The mint (ERC20 address) of the output token
    pub output_mint: Address,
    /// The amount of the input token to trade in the order, in atoms of the
    /// input token in decimal form.
    ///
    /// Conflicts with `output_amount`.
    #[serde(with = "amount_string_serde")]
    pub input_amount: Amount,
    /// The amount of the output token to trade in the order, in atoms of the
    /// output token in decimal form.
    ///
    /// Conflicts with `input_amount`.
    #[serde(with = "amount_string_serde")]
    pub output_amount: Amount,
    /// Whether the specified `output_amount` is an exact amount to receive, net
    /// of fees.
    pub use_exact_output_amount: bool,
    /// The minimum fill size for the order, in atoms of the input / output
    /// token (whichever amount was specified) in decimal form.
    ///
    /// Conflicts with `use_exact_output_amount`.
    #[serde(with = "amount_string_serde")]
    pub min_fill_size: Amount,
}

// ---------------
// | Quote Types |
// ---------------

/// A signed quote for an external order, including gas sponsorship info, if any
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedExternalQuote {
    /// The quote
    pub quote: ApiExternalQuote,
    /// The signature
    #[serde(with = "bytes_base64_serde")]
    pub signature: Vec<u8>,
    /// The deadline of the quote, in milliseconds since the epoch
    pub deadline: u64,
}

/// A quote for an external order
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiExternalQuote {
    /// The external order
    pub order: ExternalOrder,
    /// The transfer sent by the external party
    pub send: ApiExternalAssetTransfer,
    /// The transfer received by the external party, net of fees.
    pub receive: ApiExternalAssetTransfer,
    /// The estimated fees for the match
    pub fees: FeeTake,
    /// The price of the match
    pub price: ApiTimestampedPrice,
    /// The timestamp of the quote, in milliseconds since the epoch
    pub timestamp: u64,
}

/// An asset transfer from an external party
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiExternalAssetTransfer {
    /// The mint of the asset
    pub mint: Address,
    /// The amount of the asset
    #[serde(with = "amount_string_serde")]
    pub amount: Amount,
}

/// A fee take, representing the fee amounts paid to the relayer and protocol by
/// the external party when the match is settled
///
/// Fees are always paid in the output token
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct FeeTake {
    /// The amount of fees paid to the relayer
    #[serde(with = "amount_string_serde")]
    pub relayer_fee: Amount,
    /// The amount of fees paid to the protocol
    #[serde(with = "amount_string_serde")]
    pub protocol_fee: Amount,
}

/// A timestamped price
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiTimestampedPrice {
    /// The price
    #[serde(with = "f64_string_serde")]
    pub price: f64,
    /// The timestamp, in milliseconds since the epoch
    pub timestamp: u64,
}

// -------------------------
// | Gas Sponsorship Types |
// -------------------------

/// Options for requesting gas sponsorship
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GasSponsorshipOptions {
    /// Whether to disable gas sponsorship
    pub disable_gas_sponsorship: bool,
    /// The address to refund gas costs to. If unspecified, gas costs will be
    /// refunded to the sender of the match transaction.
    pub refund_address: Option<Address>,
    /// Whether to refund gas costs in terms of native ETH, as opposed to the
    /// output token of the match.
    pub refund_native_eth: bool,
}

/// Gas sponsorship applied to a quote
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GasSponsorshipInfo {
    /// The amount to be refunded as a result of gas sponsorship.
    /// This amount is firm, it will not change when the quote is assembled.
    #[serde(with = "amount_string_serde")]
    pub refund_amount: Amount,
    /// The address to which the refund will be sent, if set explicitly.
    pub refund_address: Option<Address>,
    /// Whether the refund is in terms of native ETH.
    pub refund_native_eth: bool,
}
