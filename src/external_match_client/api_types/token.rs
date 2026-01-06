//! Types for price and token endpoints

use serde::{Deserialize, Serialize};

/// A token in the the supported token list
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiToken {
    /// The token address
    pub address: String,
    /// The token symbol
    pub symbol: String,
}
