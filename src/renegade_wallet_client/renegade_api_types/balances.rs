//! Balance API types

use alloy::primitives::{Address, U256};
use renegade_circuit_types::{
    baby_jubjub::BabyJubJubPointShare,
    schnorr::{SchnorrPublicKey, SchnorrPublicKeyShare},
    Amount,
};
use renegade_constants::Scalar;
use renegade_darkpool_types::balance::{
    DarkpoolBalance, DarkpoolBalanceShare, DarkpoolStateBalance,
};
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

impl From<ApiBalance> for DarkpoolStateBalance {
    fn from(value: ApiBalance) -> Self {
        let balance = DarkpoolBalance {
            mint: value.mint,
            owner: value.owner,
            relayer_fee_recipient: value.relayer_fee_recipient,
            authority: value.authority.into(),
            amount: value.amount,
            protocol_fee_balance: value.protocol_fee_balance,
            relayer_fee_balance: value.relayer_fee_balance,
        };

        let recovery_stream = value.recovery_stream.into();
        let share_stream = value.share_stream.into();

        let public_share: DarkpoolBalanceShare = value.public_shares.into();

        DarkpoolStateBalance { recovery_stream, share_stream, inner: balance, public_share }
    }
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

impl From<ApiBalanceShare> for DarkpoolBalanceShare {
    fn from(value: ApiBalanceShare) -> Self {
        DarkpoolBalanceShare {
            mint: value.mint,
            owner: value.owner,
            relayer_fee_recipient: value.relayer_fee_recipient,
            authority: value.authority.into(),
            relayer_fee_balance: value.relayer_fee_balance,
            protocol_fee_balance: value.protocol_fee_balance,
            amount: value.amount,
        }
    }
}

/// A Schnorr public key, with custom serialization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiSchnorrPublicKey {
    /// The curve point representing the public key
    pub point: ApiBabyJubJubPoint,
}

impl From<ApiSchnorrPublicKey> for SchnorrPublicKey {
    fn from(value: ApiSchnorrPublicKey) -> Self {
        SchnorrPublicKey { point: value.point.into() }
    }
}

impl From<SchnorrPublicKey> for ApiSchnorrPublicKey {
    fn from(value: SchnorrPublicKey) -> Self {
        ApiSchnorrPublicKey { point: value.point.into() }
    }
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

impl From<ApiSchnorrPublicKeyShare> for SchnorrPublicKeyShare {
    fn from(value: ApiSchnorrPublicKeyShare) -> Self {
        SchnorrPublicKeyShare { point: BabyJubJubPointShare { x: value.x, y: value.y } }
    }
}

/// The authorization for a deposit, with custom serialization
#[derive(Clone, Debug, Serialize)]
pub struct ApiDepositPermit {
    /// The nonce that was used in the signature
    pub nonce: U256,
    /// The deadline of the permit
    pub deadline: U256,
    /// The signature bytes
    #[serde(serialize_with = "serialize_bytes_b64")]
    pub signature: Vec<u8>,
}
