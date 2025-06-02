//! Types for price and token endpoints

use serde::{Deserialize, Deserializer, Serialize};

/// A token in the the supported token list
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiToken {
    /// The token address
    pub address: String,
    /// The token symbol
    pub symbol: String,
}

/// Price information for a token
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenPrice {
    /// The mint (ERC20 address) of the base token
    pub base_token: String,
    /// The mint (ERC20 address) of the quote token
    pub quote_token: String,
    /// The price data for this token
    #[serde(deserialize_with = "string_to_f64")]
    pub price: f64,
}

// -----------
// | Helpers |
// -----------

/// Helper function to deserialize a string as f64
fn string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_price_deserialization() {
        let json = r#"{
            "base_token": "0x12345",
            "quote_token": "0x098765",
            "price": "123.456789"
        }"#;

        let token_price: TokenPrice = serde_json::from_str(json).unwrap();

        assert_eq!(token_price.base_token, "0x12345");
        assert_eq!(token_price.quote_token, "0x098765");
        assert_eq!(token_price.price, 123.456789);
    }
}
