//! Renegade external match API types

pub mod order_types;
pub mod request_response;
pub(crate) mod serde_helpers;

/// A type alias for an amount used in the Renegade system
pub type Amount = u128;
