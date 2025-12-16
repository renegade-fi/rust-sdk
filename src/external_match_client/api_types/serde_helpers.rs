//! Helpers for serializing / deserializing API types

use serde::{Deserialize, Deserializer, Serializer};

/// A module for serializing and deserializing an `Amount` as a string
pub(crate) mod amount_string_serde {
    use crate::api_types::Amount;

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
