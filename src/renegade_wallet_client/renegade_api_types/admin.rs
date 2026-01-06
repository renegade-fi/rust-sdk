//! API types for the relayer's admin API, used by the party managing the
//! relayer

use serde::Deserialize;
use uuid::Uuid;

use crate::renegade_api_types::orders::ApiOrder;

/// A Renegade order, with additional admin-relevant metadata
#[derive(Clone, Debug, Deserialize)]
pub struct ApiAdminOrder {
    /// The order itself, with all non-admin metadata
    pub order: ApiOrder,
    /// The ID of the account owning the order
    pub account_id: Uuid,
    /// The name of the matching pool to which the order is assigned
    pub matching_pool: String,
}
