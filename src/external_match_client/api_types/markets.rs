//! Types for the markets endpoints

use serde::{Deserialize, Serialize};

use crate::api_types::{
    serde_helpers::*, token::ApiToken, Amount, ApiTimestampedPrice, FeeTakeRate,
};

/// Information about a tradable market in Renegade
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketInfo {
    /// The base token
    pub base: ApiToken,
    /// The quote token
    pub quote: ApiToken,
    /// The current price of the market, in terms of quote token per base token
    pub price: ApiTimestampedPrice,
    /// The fee rates for internal matches in this market
    pub internal_match_fee_rates: FeeTakeRate,
    /// The fee rates for external matches in this market
    pub external_match_fee_rates: FeeTakeRate,
}

/// The liquidity depth for a market
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketDepth {
    /// The market information
    pub market: MarketInfo,
    /// The liquidity depth for the buy side
    pub buy: DepthSide,
    /// The liquidity depth for the sell side
    pub sell: DepthSide,
}

/// The liquidity depth for a given side of the market
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepthSide {
    /// The matchable amount at the midpoint price, in units of the base token
    #[serde(with = "amount_string_serde")]
    pub total_quantity: Amount,
    /// The matchable amount at the midpoint price, in USD
    #[serde(with = "f64_string_serde")]
    pub total_quantity_usd: f64,
}
