//! Account API types

use renegade_circuit_types::{csprng::PoseidonCSPRNG, schnorr::SchnorrPrivateKey};
use renegade_constants::EmbeddedScalarField;
use serde::{Deserialize, Serialize};

use super::serde_helpers::*;

/// A Poseidon CSPRNG's state, with custom string serialization for the seed
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

/// A Schnorr private key, with custom string serialization for the scalar field
/// element
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
