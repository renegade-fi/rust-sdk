//! HTTP request and response types

use serde::{Deserialize, Serialize};

use crate::api_types::markets::{MarketDepth, MarketInfo};

use super::{ApiSignedQuote, ExternalOrder, GasSponsorshipInfo, MalleableAtomicMatchApiBundle};

// -------------------------------
// | HTTP Requests and Responses |
// -------------------------------

/// The response type for fetching all tradable markets
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetMarketsResponse {
    /// The tradable markets
    pub markets: Vec<MarketInfo>,
}

/// The response type for fetching the market depths for all supported pairs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetMarketDepthsResponse {
    /// The market depth for all supported pairs
    pub market_depths: Vec<MarketDepth>,
}

/// The response type for fetching the market depth for a given mint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetMarketDepthByMintResponse {
    /// The market depth for the given mint
    pub market_depth: MarketDepth,
}

/// The request type for a quote on an external order
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalQuoteRequest {
    /// The external order
    pub external_order: ExternalOrder,
}

/// The response type for a quote on an external order
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalQuoteResponse {
    /// The signed quote
    pub signed_quote: ApiSignedQuote,
    /// The signed gas sponsorship info, if sponsorship was requested
    pub gas_sponsorship_info: Option<GasSponsorshipInfo>,
}

/// The request type for assembling an external match quote into a settlement
/// bundle
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssembleExternalMatchRequest {
    /// Whether or not to include gas estimation in the response
    #[serde(default)]
    pub do_gas_estimation: bool,
    /// The receiver address of the match, if not the message sender
    #[serde(default)]
    pub receiver_address: Option<String>,
    /// The type of assembly to perform
    pub order: AssemblyType,
}

/// The type of assembly to perform
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
#[allow(clippy::large_enum_variant)]
pub enum AssemblyType {
    /// Assemble a previously quoted order into a match bundle
    QuotedOrder {
        /// The signed quote
        signed_quote: ApiSignedQuote,
        /// The updated order if any changes have been made
        #[serde(default)]
        updated_order: Option<ExternalOrder>,
    },
    /// Assemble a new order into a match bundle
    NewOrder {
        /// The external order
        external_order: ExternalOrder,
    },
}

/// The response type for requesting a malleable quote on an external order
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalMatchResponse {
    /// The match bundle
    pub match_bundle: MalleableAtomicMatchApiBundle,
    /// The input amount chosen for the match
    ///
    /// If `None`, the input amount has not been selected and will default to
    /// the `max_input_amount`
    ///
    /// This field is not meant for client use directly, rather it is set by
    /// operating on the type and allows the response type to stay internally
    /// consistent
    #[serde(default)]
    pub(crate) input_amount: Option<u128>,
    /// The gas sponsorship info, if the match was sponsored
    pub gas_sponsorship_info: Option<GasSponsorshipInfo>,
}
