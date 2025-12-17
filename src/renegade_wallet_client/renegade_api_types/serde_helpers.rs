//! Helpers for serializing / deserializing API types

use renegade_common::types::hmac::HmacKey;
use serde::{Deserialize, Deserializer, Serializer};

/// A module for serializing and deserializing a `Scalar` as a decimal string
pub(crate) mod scalar_string_serde {
    use renegade_constants::Scalar;

    use super::*;

    /// Serialize a `Scalar` as a decimal string
    pub(crate) fn serialize<S>(val: &Scalar, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&val.to_string())
    }

    /// Deserialize a `Scalar` from a decimal string
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Scalar, D::Error>
    where
        D: Deserializer<'de>,
    {
        let scalar_str = String::deserialize(deserializer)?;
        Scalar::from_decimal_string(&scalar_str).map_err(serde::de::Error::custom)
    }
}

/// Serialize an `EmbeddedScalarField` as a decimal string
pub(crate) fn serialize_embedded_scalar_field<S>(
    val: &EmbeddedScalarField,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&val.to_string())
}

/// Serialize an `HmacKey` as a base64 string
pub(crate) fn serialize_hmac_key<S>(val: &HmacKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&val.to_base64_string())
}
