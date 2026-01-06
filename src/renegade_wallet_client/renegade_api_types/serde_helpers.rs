//! Helpers for serializing / deserializing API types

use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};
use renegade_constants::EmbeddedScalarField;
use serde::{Deserialize, Deserializer, Serializer};

use crate::HmacKey;

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

/// A module for serializing and deserializing an `Amount` as a string
pub(crate) mod amount_string_serde {
    use renegade_circuit_types::Amount;

    use super::*;

    /// Serialize an `Amount` as a string
    pub(crate) fn serialize<S>(val: &Amount, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&val.to_string())
    }

    /// Deserialize an `Amount` from a string
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Amount, D::Error>
    where
        D: Deserializer<'de>,
    {
        let amount_str = String::deserialize(deserializer)?;
        amount_str.parse::<Amount>().map_err(serde::de::Error::custom)
    }
}

/// A module for serializing and deserializing an `f64` as a string
pub(crate) mod f64_string_serde {
    use super::*;

    /// Serialize an `f64` as a string
    pub(crate) fn serialize<S>(val: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&val.to_string())
    }

    /// Deserialize an `f64` from a string
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let f64_str = String::deserialize(deserializer)?;
        f64_str.parse::<f64>().map_err(serde::de::Error::custom)
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

/// Serialize a `Vec<u8>` as a base64 string
pub(crate) fn serialize_bytes_b64<S>(val: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let bytes_b64 = BASE64_STANDARD_NO_PAD.encode(val);
    serializer.serialize_str(&bytes_b64)
}
