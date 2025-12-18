//! Order API types

use alloy::primitives::{Address, TxHash};
use renegade_circuit_types::{fixed_point::FixedPoint, Amount};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::serde_helpers::*;

/// A Renegade order, including metadata
#[derive(Clone, Debug, Deserialize)]
pub struct ApiOrder {
    /// The order ID
    pub id: Uuid,
    /// The mint (erc20 address) of the input token
    pub in_token: Address,
    /// The mint (erc20 address) of the output token
    pub out_token: Address,
    /// The owner of the order
    pub owner: Address,
    /// The minimum price at which the order can be filled,
    /// in units of `out_token/in_token`
    pub min_price: FixedPoint,
    /// The amount of the input token to trade
    #[serde(with = "amount_string_serde")]
    pub amount_in: Amount,
    /// The minimum fill size for the order
    #[serde(with = "amount_string_serde")]
    pub min_fill_size: Amount,
    /// The type of order
    pub order_type: OrderType,
    /// The current state of the order
    pub state: OrderState,
    /// The fills on the order
    pub fills: Vec<ApiPartialOrderFill>,
    /// The timestamp of the order's creation, in milliseconds since the epoch
    pub created: u64,
}

/// The different types of orders that can be placed in Renegade
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    /// A public order
    PublicOrder,
    /// A natively-settled, private order
    NativelySettledPrivateOrder,
    /// A Renegade-settled, public-fill order
    RenegadeSettledPublicFillOrder,
    /// A Renegade-settled, private-fill order
    RenegadeSettledPrivateFillOrder,
}

/// The different states an order can be in
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderState {
    /// The order has been created, but validity proofs are not yet proven
    Created,
    /// The order is ready to match, and is being shopped around the network
    Matching,
    /// A match has been found and is being settled
    SettlingMatch,
    /// The order has been entirely filled
    Filled,
    /// The order was canceled before it could be filled
    Cancelled,
}

/// A partial fill on an order
#[derive(Clone, Debug, Deserialize)]
pub struct ApiPartialOrderFill {
    /// The amount of the input token that was filled
    #[serde(with = "amount_string_serde")]
    pub amount: Amount,
    /// The price at which the fill was executed, in units of
    /// `quote_token/base_token`.
    ///
    /// Renegade always uses USDC as the quote token.
    pub price: ApiTimestampedPrice,
    /// The fees paid on the fill, in units of the output token
    pub fees: FeeTake,
    /// The hash of the fill transaction
    pub tx_hash: TxHash,
}

/// A timestamped price

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiTimestampedPrice {
    /// The price, in units of `quote_token/base_token`.
    ///
    /// Renegade always uses USDC as the quote token.
    #[serde(with = "f64_string_serde")]
    pub price: f64,
    /// The timestamp of the price, in milliseconds since the epoch
    pub timestamp: u64,
}

/// A fee take, representing the fee amounts paid to the relayer and protocol
/// when a match is settled
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
