//! API types for the relayer's admin API, used by the party managing the
//! relayer

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::renegade_api_types::orders::{ApiOrder, ApiOrderCore};

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

/// A Renegade order core with matching pool (admin variant)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiAdminOrderCore {
    /// The order core
    #[serde(flatten)]
    pub order_core: ApiOrderCore,
    /// The matching pool to assign the order to
    pub matching_pool: String,
}
