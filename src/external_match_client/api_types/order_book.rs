//! Types for the order book endpoints
use serde::{Deserialize, Serialize};

/// Response for the GET /order_book/depth/:mint route
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetDepthByMintResponse {
    /// The current price of the token in USD
    pub price: f64,
    /// The timestamp of the price
    pub timestamp: u64,
    /// The buy side depth
    pub buy: DepthSide,
    /// The sell side depth
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
