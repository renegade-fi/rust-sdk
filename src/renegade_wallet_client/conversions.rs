//! Conversion helpers between upstream API types and internal darkpool types
//!
//! The upstream `renegade-external-api` crate now provides `From`/`Into` impls
//! for most conversions. This module re-exports the conversions that require
//! error handling (i.e. `TryFrom` impls).

use renegade_darkpool_types::balance::DarkpoolStateBalance;
use renegade_external_api::types::ApiBalance;

use crate::RenegadeClientError;

/// Convert an `ApiBalance` to a `DarkpoolStateBalance`
pub fn api_balance_to_state_balance(
    api_balance: ApiBalance,
) -> Result<DarkpoolStateBalance, RenegadeClientError> {
    api_balance.try_into().map_err(RenegadeClientError::custom)
}
