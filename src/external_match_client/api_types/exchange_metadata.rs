//! Types for the exchange metadata endpoints

use serde::{Deserialize, Serialize};

use crate::api_types::token::ApiToken;

/// The metadata for the Renegade exchange
///
/// This type is used to get metadata about Renegade
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExchangeMetadataResponse {
    /// The chain id of the connected
    pub chain_id: u64,
    /// The address of the settlement contract
    pub settlement_contract_address: String,
    /// The supported tokens
    pub supported_tokens: Vec<ApiToken>,
}
