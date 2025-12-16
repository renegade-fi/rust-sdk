//! Top-level HTTP request & response types for the Renegade external match API

use serde::{Deserialize, Serialize};

use crate::v2::external_match_client::api::order_types::{
    ExternalOrder, GasSponsorshipInfo, GasSponsorshipOptions, SignedExternalQuote,
};

// ---------------------------------
// | v2/external-matches/get-quote |
// ---------------------------------

/// The request type for a quote on an external order
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalQuoteRequest {
    /// The external order
    pub external_order: ExternalOrder,
    /// The gas sponsorship options
    #[serde(default)]
    pub gas_sponsorship: GasSponsorshipOptions,
}

/// The response type for a quote on an external order
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalQuoteResponse {
    /// The signed quote
    pub signed_quote: SignedExternalQuote,
    /// The gas sponsorship info, if sponsorship was applied
    pub gas_sponsorship_info: Option<GasSponsorshipInfo>,
}
