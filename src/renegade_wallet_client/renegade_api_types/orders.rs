//! Order API types

use alloy::primitives::{Address, TxHash, U256};
use renegade_circuit_types::{fixed_point::FixedPoint, Amount};
use renegade_solidity_abi::v2::{relayer_types::u128_to_u256, IDarkpoolV2};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::renegade_api_types::account::ApiSchnorrSignature;

use super::serde_helpers::*;

/// A Renegade order, without metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiOrderCore {
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
    /// Whether to allow external matches on the order
    pub allow_external_matches: bool,
}

impl From<&ApiOrderCore> for IDarkpoolV2::Intent {
    fn from(value: &ApiOrderCore) -> Self {
        IDarkpoolV2::Intent {
            inToken: value.in_token,
            outToken: value.out_token,
            owner: value.owner,
            minPrice: value.min_price.into(),
            amountIn: u128_to_u256(value.amount_in),
        }
    }
}

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
    /// Whether to allow external matches on the order
    pub allow_external_matches: bool,
    /// The current state of the order
    pub state: OrderState,
    /// The fills on the order
    pub fills: Vec<ApiPartialOrderFill>,
    /// The timestamp of the order's creation, in milliseconds since the epoch
    pub created: u64,
}

impl From<ApiOrder> for ApiOrderCore {
    fn from(value: ApiOrder) -> Self {
        Self {
            id: value.id,
            in_token: value.in_token,
            out_token: value.out_token,
            owner: value.owner,
            min_price: value.min_price,
            amount_in: value.amount_in,
            min_fill_size: value.min_fill_size,
            order_type: value.order_type,
            allow_external_matches: value.allow_external_matches,
        }
    }
}

/// The different types of orders that can be placed in Renegade
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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

/// The authorization methods for each order type
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderAuth {
    /// A public order
    PublicOrder {
        /// The signature over the intent
        intent_signature: SignatureWithNonce,
    },
    /// A natively-settled, private order
    NativelySettledPrivateOrder {
        /// The Schnorr signature over the intent
        intent_signature: ApiSchnorrSignature,
    },
    /// A Renegade-settled order
    RenegadeSettledOrder {
        /// The Schnorr signature over the intent
        intent_signature: ApiSchnorrSignature,
        /// The Schnorr signature over a new output balance,
        /// in case one needs to be created
        new_output_balance_signature: ApiSchnorrSignature,
    },
}

/// A signature with a nonce
#[derive(Clone, Debug, Serialize)]
pub struct SignatureWithNonce {
    /// The nonce that was used in the signature
    pub nonce: U256,
    /// The signature bytes
    #[serde(serialize_with = "serialize_bytes_b64")]
    pub signature: Vec<u8>,
}

impl From<IDarkpoolV2::SignatureWithNonce> for SignatureWithNonce {
    fn from(value: IDarkpoolV2::SignatureWithNonce) -> Self {
        Self { nonce: value.nonce, signature: value.signature.into() }
    }
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
