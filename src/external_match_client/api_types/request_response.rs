//! HTTP request and response types

use serde::{Deserialize, Serialize};

use crate::api_types::token::TokenPrice;

use super::{
    token::ApiToken, ApiSignedQuote, AtomicMatchApiBundle, ExternalOrder, GasSponsorshipInfo,
    MalleableAtomicMatchApiBundle, SignedGasSponsorshipInfo,
};

// -------------------------------
// | HTTP Requests and Responses |
// -------------------------------

/// The response type to fetch the supported token list
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetSupportedTokensResponse {
    /// The supported tokens
    pub tokens: Vec<ApiToken>,
}

/// The response type to fetch the token prices
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetTokenPricesResponse {
    /// The token prices
    pub token_prices: Vec<TokenPrice>,
}

/// The request type for requesting an external match
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalMatchRequest {
    /// Whether or not to include gas estimation in the response
    #[serde(default)]
    pub do_gas_estimation: bool,
    /// The receiver address of the match, if not the message sender
    #[serde(default)]
    pub receiver_address: Option<String>,
    /// The external order
    pub external_order: ExternalOrder,
}

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
    /// The gas sponsorship info, if the match was sponsored
    pub gas_sponsorship_info: Option<GasSponsorshipInfo>,
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
    pub gas_sponsorship_info: Option<SignedGasSponsorshipInfo>,
}

/// The request type for assembling an external match quote into a settlement
/// bundle
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssembleExternalMatchRequest {
    /// Whether or not to include gas estimation in the response
    #[serde(default)]
    pub do_gas_estimation: bool,
    /// Whether or not to allow shared access to the resulting bundle
    ///
    /// If true, the bundle may be sent to other clients requesting an external
    /// match. If false, the bundle will be exclusively held for some time
    #[serde(default)]
    pub allow_shared: bool,
    /// The receiver address of the match, if not the message sender
    #[serde(default)]
    pub receiver_address: Option<String>,
    /// The updated order if any changes have been made
    #[serde(default)]
    pub updated_order: Option<ExternalOrder>,
    /// The signed quote
    pub signed_quote: ApiSignedQuote,
}

/// A type alias for the malleable match response routed through the gas sponsor
/// ABI
pub type MalleableExternalMatchResponse =
    GenericMalleableExternalMatchResponse<false /* USE_CONNECTOR */>;
/// A type alias for the malleable match response routed through the connector
/// ABI
pub type MalleableExternalMatchResponseWithConnector =
    GenericMalleableExternalMatchResponse<true /* USE_CONNECTOR */>;

/// The response type for requesting a malleable quote on an external order
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenericMalleableExternalMatchResponse<const USE_CONNECTOR: bool> {
    /// The match bundle
    pub match_bundle: MalleableAtomicMatchApiBundle,
    /// The base amount chosen for the match
    ///
    /// If `None`, the base amount has not been selected and will default to the
    /// `max_base_amount`
    ///
    /// This field is not meant for client use directly, rather it is set by
    /// operating on the type and allows the response type to stay internally
    /// consistent
    #[serde(default)]
    pub(crate) base_amount: Option<u128>,
    /// The quote amount chosen for the match
    ///
    /// If `None`, the quote amount has not been selected and will default to
    /// the quote amount implied by the `max_base_amount` and the price in
    /// the match result.
    ///
    /// This field is not meant for client use directly, rather it is set by
    /// operating on the type and allows the response type to stay internally
    /// consistent
    #[serde(default)]
    pub(crate) quote_amount: Option<u128>,
    /// The gas sponsorship info, if the match was sponsored
    pub gas_sponsorship_info: Option<GasSponsorshipInfo>,
}
