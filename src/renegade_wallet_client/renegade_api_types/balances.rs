//! Balance API types

use alloy::primitives::Address;
use renegade_circuit_types::Amount;
use renegade_constants::Scalar;
use serde::{Deserialize, Serialize};

use crate::renegade_api_types::account::{ApiBabyJubJubPoint, ApiPoseidonCSPRNG};

use super::serde_helpers::*;

/// A Renegade balance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiBalance {
    /// The mint of the token in the balance
    pub mint: Address,
    /// The owner of the balance
    pub owner: Address,
    /// The address to which the relayer fees are paid
    pub relayer_fee_recipient: Address,
    /// The public key which authorizes the creation of new state elements
    pub authority: ApiSchnorrPublicKey,
    /// The relayer fee balance of the balance
    #[serde(with = "amount_string_serde")]
    pub relayer_fee_balance: Amount,
    /// The protocol fee balance of the balance
    #[serde(with = "amount_string_serde")]
    pub protocol_fee_balance: Amount,
    /// The amount of the token in the balance
    #[serde(with = "amount_string_serde")]
    pub amount: Amount,
    /// The recovery stream for the balance
    pub recovery_stream: ApiPoseidonCSPRNG,
    /// The share stream for the balance
    pub share_stream: ApiPoseidonCSPRNG,
    /// The public sharing of the balance
    pub public_shares: ApiBalanceShare,
}

/// A public sharing of a balance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiBalanceShare {
    /// The public sharing of the mint
    #[serde(with = "scalar_string_serde")]
    pub mint: Scalar,
    /// The public sharing of the owner
    #[serde(with = "scalar_string_serde")]
    pub owner: Scalar,
    /// The public sharing of the relayer fee recipient
    #[serde(with = "scalar_string_serde")]
    pub relayer_fee_recipient: Scalar,
    /// The public sharing of the authority
    pub authority: ApiSchnorrPublicKeyShare,
    /// The public sharing of the relayer fee balance
    #[serde(with = "scalar_string_serde")]
    pub relayer_fee_balance: Scalar,
    /// The public sharing of the protocol fee balance
    #[serde(with = "scalar_string_serde")]
    pub protocol_fee_balance: Scalar,
    /// The public sharing of the amount
    #[serde(with = "scalar_string_serde")]
    pub amount: Scalar,
}

/// A Schnorr public key, with custom serialization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiSchnorrPublicKey {
    /// The curve point representing the public key
    pub point: ApiBabyJubJubPoint,
}

/// A public sharing of a Schnorr public key, with custom serialization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiSchnorrPublicKeyShare {
    /// The x coordinate of the public key point
    #[serde(with = "scalar_string_serde")]
    pub x: Scalar,
    /// The y coordinate of the public key point
    #[serde(with = "scalar_string_serde")]
    pub y: Scalar,
}
