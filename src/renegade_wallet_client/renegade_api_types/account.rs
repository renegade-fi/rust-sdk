//! Account API types

use renegade_circuit_types::{
    csprng::PoseidonCSPRNG,
    elgamal::BabyJubJubPoint,
    schnorr::{SchnorrPrivateKey, SchnorrSignature},
};
use renegade_constants::{EmbeddedScalarField, Scalar};
use serde::{Deserialize, Serialize};

use super::serde_helpers::*;

/// A Poseidon CSPRNG's state, with custom serialization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiPoseidonCSPRNG {
    /// The seed of the CSPRNG
    #[serde(with = "scalar_string_serde")]
    pub seed: Scalar,
    /// The index into the CSPRNG's stream
    pub index: u64,
}

impl From<ApiPoseidonCSPRNG> for PoseidonCSPRNG {
    fn from(value: ApiPoseidonCSPRNG) -> Self {
        PoseidonCSPRNG { seed: value.seed, index: value.index }
    }
}

/// A Schnorr private key, with custom serialization
#[derive(Copy, Clone, Debug, Serialize)]
pub struct ApiSchnorrPrivateKey {
    /// The underlying scalar field element
    #[serde(serialize_with = "serialize_embedded_scalar_field")]
    pub inner: EmbeddedScalarField,
}

impl From<ApiSchnorrPrivateKey> for SchnorrPrivateKey {
    fn from(value: ApiSchnorrPrivateKey) -> Self {
        SchnorrPrivateKey { inner: value.inner }
    }
}

impl From<SchnorrPrivateKey> for ApiSchnorrPrivateKey {
    fn from(value: SchnorrPrivateKey) -> Self {
        Self { inner: value.inner }
    }
}

/// A Schnorr signature, with custom serialization
#[derive(Copy, Clone, Debug, Serialize)]
pub struct ApiSchnorrSignature {
    /// The s-value of the signature
    ///
    /// s = H(M || r) * private_key + k
    #[serde(serialize_with = "serialize_embedded_scalar_field")]
    pub s: EmbeddedScalarField,
    /// The R-value of the signature
    ///
    /// r = k * G for random k; though practically k is made deterministic via
    /// the transcript.
    pub r: ApiBabyJubJubPoint,
}

impl From<SchnorrSignature> for ApiSchnorrSignature {
    fn from(value: SchnorrSignature) -> Self {
        Self { s: value.s, r: value.r.into() }
    }
}

/// A BabyJubJub point, with custom serialization
#[derive(Copy, Clone, Debug, Serialize)]
pub struct ApiBabyJubJubPoint {
    /// The x coordinate of the point
    #[serde(with = "scalar_string_serde")]
    pub x: Scalar,
    /// The y coordinate of the point
    #[serde(with = "scalar_string_serde")]
    pub y: Scalar,
}

impl From<BabyJubJubPoint> for ApiBabyJubJubPoint {
    fn from(value: BabyJubJubPoint) -> Self {
        Self { x: value.x, y: value.y }
    }
}
