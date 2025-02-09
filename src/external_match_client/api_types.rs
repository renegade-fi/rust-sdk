//! Types for the external match client

use renegade_api::http::external_match::AtomicMatchApiBundle;
use serde::{Deserialize, Serialize};

/// The response type for requesting an external match
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalMatchResponse {
    /// The raw response from the relayer
    pub match_bundle: AtomicMatchApiBundle,
    /// Whether the match has received gas sponsorship
    ///
    /// If `true`, the bundle is routed through a gas rebate contract that
    /// refunds the gas used by the match to the configured address
    #[serde(rename = "is_sponsored", default)]
    pub gas_sponsored: bool,
}
