//! Types for the order book endpoints
use serde::{Deserialize, Serialize};

/// Response for the GET /order_book/depth/:mint route
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetDepthByMintResponse {
    /// The liquidity depth for the given mint
    #[serde(flatten)]
    pub depth: PriceAndDepth,
}

/// Response for the GET /order_book/depth route
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetDepthForAllPairsResponse {
    /// The liquidity depth for all supported pairs
    pub pairs: Vec<PriceAndDepth>,
}

/// The liquidity depth for a given pair
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PriceAndDepth {
    /// The token address
    pub address: String,
    /// The current price of the token in USD
    pub price: f64,
    /// The timestamp of the price
    pub timestamp: u64,
    /// The liquidity depth for the buy side
    pub buy: DepthSide,
    /// The liquidity depth for the sell side
    pub sell: DepthSide,
}

/// The liquidity depth for a given side of the market
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DepthSide {
    /// The matchable amount at the midpoint price, in units of the base token
    pub total_quantity: u128,
    /// The matchable amount at the midpoint price, in USD
    pub total_quantity_usd: f64,
}
