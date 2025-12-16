//! Helpers for serializing / deserializing API types

use serde::{Deserialize, Deserializer, Serializer};

/// A module for serializing and deserializing an `Amount` as a string
pub(crate) mod amount_string_serde {
    use super::*;
    use crate::v2::external_match_client::api::Amount;

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

    /// Serialize a `f64` as a string
    pub(crate) fn serialize<S>(val: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&val.to_string())
    }

    /// Deserialize a `f64` from a string
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        let f64_str = String::deserialize(deserializer)?;
        f64_str.parse::<f64>().map_err(serde::de::Error::custom)
    }
}

/// A module for serializing and deserializing a `Vec<u8>` as a base64 string
pub(crate) mod bytes_base64_serde {
    use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};

    use super::*;

    /// Serialize a `Vec<u8>` as a base64 string
    pub(crate) fn serialize<S>(val: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes_b64 = BASE64_STANDARD_NO_PAD.encode(val);
        serializer.serialize_str(&bytes_b64)
    }

    /// Deserialize a `Vec<u8>` from a base64 string
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes_b64 = String::deserialize(deserializer)?;
        BASE64_STANDARD_NO_PAD.decode(bytes_b64).map_err(serde::de::Error::custom)
    }
}
