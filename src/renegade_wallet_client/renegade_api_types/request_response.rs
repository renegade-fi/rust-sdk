//! Top-level API request / response types

use alloy::primitives::Address;
use renegade_constants::Scalar;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    renegade_api_types::{
        account::{ApiPoseidonCSPRNG, ApiSchnorrPrivateKey},
        orders::ApiOrder,
    },
    HmacKey,
};

use super::serde_helpers::*;

// ---------------
// | Account API |
// ---------------

/// A request to create an account
#[derive(Debug, Serialize)]
pub struct CreateAccountRequest {
    /// The account ID
    pub account_id: Uuid,
    /// The address of the account owner
    pub address: Address,
    /// The master view seed
    #[serde(with = "scalar_string_serde")]
    pub master_view_seed: Scalar,
    /// The Schnorr private key
    pub schnorr_key: ApiSchnorrPrivateKey,
    /// The HMAC key for API authentication
    #[serde(serialize_with = "serialize_hmac_key")]
    pub auth_hmac_key: HmacKey,
}

/// A response containing the current states of an account's
/// seed CSPRNGs
#[derive(Debug, Deserialize)]
pub struct GetAccountSeedsResponse {
    /// The current state of the recovery stream seeds CSPRNG
    pub recovery_seed_csprng: ApiPoseidonCSPRNG,
    /// The current state of the share stream seeds CSPRNG
    pub share_seed_csprng: ApiPoseidonCSPRNG,
}

// # === Orders === #

/// The query parameters used when fetching all account orders
#[derive(Debug, Default, Serialize)]
pub struct GetOrdersQueryParameters {
    /// Whether to include historic (inactive) orders in the response
    pub include_historic_orders: Option<bool>,
    /// The number of orders to return per page
    pub page_size: Option<usize>,
    /// The page token to use for pagination
    pub page_token: Option<usize>,
}

/// A response containing a page of orders
#[derive(Debug, Deserialize)]
pub struct GetOrdersResponse {
    /// The orders
    pub orders: Vec<ApiOrder>,
    /// The next page token to use for pagination, if more orders are available
    pub next_page_token: Option<usize>,
}

/// A response containing a single order
#[derive(Debug, Deserialize)]
pub struct GetOrderByIdResponse {
    /// The order
    pub order: ApiOrder,
}
