//! The client for requesting external matches

use reqwest::{
    header::{HeaderMap, HeaderValue},
    StatusCode,
};
use url::form_urlencoded;

use crate::{
    api_types::{
        ASSEMBLE_EXTERNAL_MATCH_MALLEABLE_ROUTE, ASSEMBLE_EXTERNAL_MATCH_ROUTE,
        REQUEST_EXTERNAL_MATCH_ROUTE,
    },
    http::RelayerHttpClient,
    util::HmacKey,
    GAS_REFUND_NATIVE_ETH_QUERY_PARAM,
};

use super::{
    api_types::{
        ApiSignedQuote, AssembleExternalMatchRequest, ExternalMatchRequest, ExternalMatchResponse,
        ExternalOrder, ExternalQuoteRequest, ExternalQuoteResponse, GetSupportedTokensResponse,
        MalleableExternalMatchResponse, SignedExternalQuote, GET_SUPPORTED_TOKENS_ROUTE,
        REQUEST_EXTERNAL_QUOTE_ROUTE,
    },
    error::ExternalMatchClientError,
    GAS_REFUND_ADDRESS_QUERY_PARAM, GAS_SPONSORSHIP_QUERY_PARAM,
};

// -------------
// | Constants |
// -------------

/// The Renegade API key header
pub const RENEGADE_API_KEY_HEADER: &str = "X-Renegade-Api-Key";

/// The Arbitrum Sepolia auth server base URL
const ARBITRUM_SEPOLIA_AUTH_BASE_URL: &str = "https://arbitrum-sepolia.auth-server.renegade.fi";
/// The Arbitrum One auth server base URL
const ARBITRUM_ONE_AUTH_BASE_URL: &str = "https://arbitrum-one.auth-server.renegade.fi";
/// The Base Sepolia auth server base URL
const BASE_SEPOLIA_AUTH_BASE_URL: &str = "https://base-sepolia.auth-server.renegade.fi";
/// The Base mainnet auth server base URL
const BASE_MAINNET_AUTH_BASE_URL: &str = "https://base-mainnet.auth-server.renegade.fi";
/// The Arbitrum Sepolia relayer base URL
const ARBITRUM_SEPOLIA_RELAYER_BASE_URL: &str = "https://arbitrum-sepolia.relayer.renegade.fi";
/// The Arbitrum One relayer base URL
const ARBITRUM_ONE_RELAYER_BASE_URL: &str = "https://arbitrum-one.relayer.renegade.fi";
/// The Base Sepolia relayer base URL
const BASE_SEPOLIA_RELAYER_BASE_URL: &str = "https://base-sepolia.relayer.renegade.fi";
/// The Base mainnet relayer base URL
const BASE_MAINNET_RELAYER_BASE_URL: &str = "https://base-mainnet.relayer.renegade.fi";

// -------------------
// | Request Options |
// -------------------

/// The options for requesting a quote
#[derive(Clone, Default)]
pub struct RequestQuoteOptions {
    /// Whether to disable gas sponsorship
    pub disable_gas_sponsorship: bool,
    /// The address to refund gas to if `sponsor_gas` is true
    pub gas_refund_address: Option<String>,
    /// Whether to refund gas in terms of native ETH, as opposed to in-kind
    pub refund_native_eth: bool,
}

impl RequestQuoteOptions {
    /// Create a new options with default values
    pub fn new() -> Self {
        Default::default()
    }

    /// Disable gas sponsorship
    pub fn disable_gas_sponsorship(mut self) -> Self {
        self.disable_gas_sponsorship = true;
        self
    }

    /// Set the gas refund address
    pub fn with_gas_refund_address(mut self, gas_refund_address: String) -> Self {
        self.gas_refund_address = Some(gas_refund_address);
        self
    }

    /// Set whether to refund gas in terms of native ETH
    pub fn with_refund_native_eth(mut self) -> Self {
        self.refund_native_eth = true;
        self
    }

    /// Get the request path given the options
    pub(crate) fn build_request_path(&self) -> String {
        let mut query = form_urlencoded::Serializer::new(String::new());
        query.append_pair(GAS_SPONSORSHIP_QUERY_PARAM, &self.disable_gas_sponsorship.to_string());
        query.append_pair(GAS_REFUND_NATIVE_ETH_QUERY_PARAM, &self.refund_native_eth.to_string());

        if let Some(addr) = &self.gas_refund_address {
            query.append_pair(GAS_REFUND_ADDRESS_QUERY_PARAM, addr);
        }

        format!("{}?{}", REQUEST_EXTERNAL_QUOTE_ROUTE, query.finish())
    }
}

/// The options for requesting an external match
#[deprecated(
    since = "0.1.0",
    note = "This endpoint will soon be removed, use `request_quote` and `assemble_quote` instead"
)]
#[derive(Clone, Default)]
pub struct ExternalMatchOptions {
    /// Whether to perform gas estimation
    pub do_gas_estimation: bool,
    /// Whether or not to request gas sponsorship for the match
    ///
    /// If granted, the auth server will sign the bundle to indicate that the
    /// gas paid to settle the match should be refunded to the given address
    /// (`tx.origin` if not specified). This is subject to a rate limit.
    pub sponsor_gas: bool,
    /// The address to refund gas to if `sponsor_gas` is true
    pub gas_refund_address: Option<String>,
    /// The receiver address that the darkpool will send funds to
    ///
    /// If not provided, the receiver address is the message sender
    pub receiver_address: Option<String>,
}

#[allow(deprecated)]
impl ExternalMatchOptions {
    /// Create a new options with default values
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the gas estimation flag
    pub fn with_gas_estimation(mut self, do_gas_estimation: bool) -> Self {
        self.do_gas_estimation = do_gas_estimation;
        self
    }

    /// Set the receiver address
    pub fn with_receiver_address(mut self, receiver_address: String) -> Self {
        self.receiver_address = Some(receiver_address);
        self
    }

    /// Request gas sponsorship
    pub fn request_gas_sponsorship(mut self) -> Self {
        self.sponsor_gas = true;
        self
    }

    /// Set the gas refund address
    pub fn with_gas_refund_address(mut self, gas_refund_address: String) -> Self {
        self.gas_refund_address = Some(gas_refund_address);
        self
    }

    /// Get the request path given the options
    pub(crate) fn build_request_path(&self) -> String {
        let mut query = form_urlencoded::Serializer::new(String::new());

        // Add query params for gas sponsorship
        query.append_pair(GAS_SPONSORSHIP_QUERY_PARAM, &(!self.sponsor_gas).to_string());
        if let Some(addr) = &self.gas_refund_address {
            query.append_pair(GAS_REFUND_ADDRESS_QUERY_PARAM, addr);
        }

        format!("{}?{}", REQUEST_EXTERNAL_MATCH_ROUTE, query.finish())
    }
}

/// The options for assembling a quote
#[derive(Clone, Default)]
pub struct AssembleQuoteOptions {
    /// Whether to do gas estimation
    pub do_gas_estimation: bool,
    /// Whether or not to allow shared access to the resulting bundle
    ///
    /// If true, the bundle may be sent to other clients requesting an external
    /// match. If false, the bundle will be exclusively held for some time
    pub allow_shared: bool,
    /// Whether or not to request gas sponsorship for the match
    ///
    /// If granted, the auth server will sign the bundle to indicate that the
    /// gas paid to settle the match should be refunded to the given address
    /// (`tx.origin` if not specified). This is subject to a rate limit.
    #[deprecated(
        since = "0.1.0",
        note = "This option will soon be removed, request gas sponsorship when requesting a quote instead"
    )]
    pub sponsor_gas: bool,
    /// The address to refund gas to if `sponsor_gas` is true
    #[deprecated(
        since = "0.1.0",
        note = "This option will soon be removed, request gas sponsorship when requesting a quote instead"
    )]
    pub gas_refund_address: Option<String>,
    /// The receiver address that the darkpool will send funds to
    ///
    /// If not provided, the receiver address is the message sender
    pub receiver_address: Option<String>,
    /// The updated order to use when assembling the quote
    ///
    /// The `base_amount`, `quote_amount`, and `min_fill_size` are allowed to
    /// change, but the pair and side is not
    pub updated_order: Option<ExternalOrder>,
}

impl AssembleQuoteOptions {
    /// Create a new options with default values
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the gas estimation flag
    pub fn with_gas_estimation(mut self, do_gas_estimation: bool) -> Self {
        self.do_gas_estimation = do_gas_estimation;
        self
    }

    /// Set the allow shared flag
    pub fn with_allow_shared(mut self, allow_shared: bool) -> Self {
        self.allow_shared = allow_shared;
        self
    }

    /// Request gas sponsorship
    #[deprecated(
        since = "0.1.0",
        note = "This option will soon be removed, request gas sponsorship when requesting a quote instead"
    )]
    #[allow(deprecated)]
    pub fn request_gas_sponsorship(mut self) -> Self {
        self.sponsor_gas = true;
        self
    }

    /// Set the gas refund address
    #[deprecated(
        since = "0.1.0",
        note = "This option will soon be removed, request gas sponsorship when requesting a quote instead"
    )]
    #[allow(deprecated)]
    pub fn with_gas_refund_address(mut self, gas_refund_address: String) -> Self {
        self.gas_refund_address = Some(gas_refund_address);
        self
    }

    /// Set the receiver address
    pub fn with_receiver_address(mut self, receiver_address: String) -> Self {
        self.receiver_address = Some(receiver_address);
        self
    }

    /// Set the updated order
    pub fn with_updated_order(mut self, updated_order: ExternalOrder) -> Self {
        self.updated_order = Some(updated_order);
        self
    }

    /// Get the request path given the options
    #[allow(deprecated)]
    pub(crate) fn build_request_path(&self) -> String {
        let mut query = form_urlencoded::Serializer::new(String::new());
        if self.sponsor_gas {
            // We only write this query parameter if it was explicitly set. The
            // expectation of the auth server is that when gas sponsorship is
            // requested at the quote stage, there should be no query parameters
            // at all in the assemble request.
            query.append_pair(GAS_SPONSORSHIP_QUERY_PARAM, &(!self.sponsor_gas).to_string());
        }

        if let Some(addr) = &self.gas_refund_address {
            query.append_pair(GAS_REFUND_ADDRESS_QUERY_PARAM, addr);
        }

        let query_str = query.finish();
        if query_str.is_empty() {
            return ASSEMBLE_EXTERNAL_MATCH_ROUTE.to_string();
        }

        format!("{ASSEMBLE_EXTERNAL_MATCH_ROUTE}?{query_str}")
    }
}

// ----------
// | Client |
// ----------

/// A client for requesting external matches from the relayer
#[derive(Clone)]
pub struct ExternalMatchClient {
    /// The api key for the external match client
    api_key: String,
    /// The HTTP client
    auth_http_client: RelayerHttpClient,
    /// The relayer HTTP client
    ///
    /// Separate from the auth client as they request different base URLs
    relayer_http_client: RelayerHttpClient,
}

impl ExternalMatchClient {
    /// Create a new client
    pub fn new(
        api_key: &str,
        api_secret: &str,
        auth_base_url: &str,
        relayer_base_url: &str,
    ) -> Result<Self, ExternalMatchClientError> {
        let api_secret = HmacKey::from_base64_string(api_secret)
            .map_err(|_| ExternalMatchClientError::InvalidApiSecret)?;

        Ok(Self {
            api_key: api_key.to_string(),
            auth_http_client: RelayerHttpClient::new(auth_base_url.to_string(), api_secret),
            relayer_http_client: RelayerHttpClient::new(relayer_base_url.to_string(), api_secret),
        })
    }

    /// Create a new client for the Arbitrum Sepolia network
    pub fn new_arbitrum_sepolia_client(
        api_key: &str,
        api_secret: &str,
    ) -> Result<Self, ExternalMatchClientError> {
        Self::new(
            api_key,
            api_secret,
            ARBITRUM_SEPOLIA_AUTH_BASE_URL,
            ARBITRUM_SEPOLIA_RELAYER_BASE_URL,
        )
    }

    /// Create a new client for the Base Sepolia network
    pub fn new_base_sepolia_client(
        api_key: &str,
        api_secret: &str,
    ) -> Result<Self, ExternalMatchClientError> {
        Self::new(api_key, api_secret, BASE_SEPOLIA_AUTH_BASE_URL, BASE_SEPOLIA_RELAYER_BASE_URL)
    }

    /// Create a new client for the Arbitrum Sepolia network
    #[deprecated(since = "0.1.6", note = "Use new_arbitrum_sepolia_client instead")]
    pub fn new_sepolia_client(
        api_key: &str,
        api_secret: &str,
    ) -> Result<Self, ExternalMatchClientError> {
        Self::new_arbitrum_sepolia_client(api_key, api_secret)
    }

    /// Create a new client for the Arbitrum One network
    pub fn new_arbitrum_one_client(
        api_key: &str,
        api_secret: &str,
    ) -> Result<Self, ExternalMatchClientError> {
        Self::new(api_key, api_secret, ARBITRUM_ONE_AUTH_BASE_URL, ARBITRUM_ONE_RELAYER_BASE_URL)
    }

    /// Create a new client for the Base mainnet network
    pub fn new_base_mainnet_client(
        api_key: &str,
        api_secret: &str,
    ) -> Result<Self, ExternalMatchClientError> {
        Self::new(api_key, api_secret, BASE_MAINNET_AUTH_BASE_URL, BASE_MAINNET_RELAYER_BASE_URL)
    }

    /// Create a new client for the Arbitrum One network
    #[deprecated(since = "0.1.6", note = "Use new_arbitrum_one_client instead")]
    pub fn new_mainnet_client(
        api_key: &str,
        api_secret: &str,
    ) -> Result<Self, ExternalMatchClientError> {
        Self::new_arbitrum_one_client(api_key, api_secret)
    }

    /// Get a list of supported tokens for external matches    
    pub async fn get_supported_tokens(
        &self,
    ) -> Result<GetSupportedTokensResponse, ExternalMatchClientError> {
        let path = GET_SUPPORTED_TOKENS_ROUTE;
        let resp = self.relayer_http_client.get(path).await?;

        Ok(resp)
    }

    /// Request a quote for an external match
    pub async fn request_quote(
        &self,
        order: ExternalOrder,
    ) -> Result<Option<SignedExternalQuote>, ExternalMatchClientError> {
        self.request_quote_with_options(order, RequestQuoteOptions::default()).await
    }

    /// Request a quote for an external match, with options
    pub async fn request_quote_with_options(
        &self,
        order: ExternalOrder,
        options: RequestQuoteOptions,
    ) -> Result<Option<SignedExternalQuote>, ExternalMatchClientError> {
        let request = ExternalQuoteRequest { external_order: order };
        let path = options.build_request_path();
        let headers = self.get_headers()?;

        let resp = self.auth_http_client.post_with_headers_raw(&path, request, headers).await?;
        let quote_resp = Self::handle_optional_response::<ExternalQuoteResponse>(resp).await?;
        Ok(quote_resp.map(|r| {
            let ApiSignedQuote { quote, signature } = r.signed_quote;
            SignedExternalQuote { quote, signature, gas_sponsorship_info: r.gas_sponsorship_info }
        }))
    }

    /// Assemble a quote into a match bundle, ready for settlement
    pub async fn assemble_quote(
        &self,
        quote: SignedExternalQuote,
    ) -> Result<Option<ExternalMatchResponse>, ExternalMatchClientError> {
        self.assemble_quote_with_options(quote, AssembleQuoteOptions::default()).await
    }

    /// Assemble a quote into a match bundle, ready for settlement, with options
    pub async fn assemble_quote_with_options(
        &self,
        quote: SignedExternalQuote,
        options: AssembleQuoteOptions,
    ) -> Result<Option<ExternalMatchResponse>, ExternalMatchClientError> {
        let path = options.build_request_path();
        let signed_quote = ApiSignedQuote { quote: quote.quote, signature: quote.signature };
        let request = AssembleExternalMatchRequest {
            signed_quote,
            receiver_address: options.receiver_address,
            do_gas_estimation: options.do_gas_estimation,
            allow_shared: options.allow_shared,
            updated_order: options.updated_order,
        };
        let headers = self.get_headers()?;

        let resp =
            self.auth_http_client.post_with_headers_raw(path.as_str(), request, headers).await?;
        let match_resp = Self::handle_optional_response::<ExternalMatchResponse>(resp).await?;
        Ok(match_resp)
    }

    /// Assemble a quote into a malleable match bundle, ready for settlement
    pub async fn assemble_malleable_quote(
        &self,
        quote: SignedExternalQuote,
    ) -> Result<Option<MalleableExternalMatchResponse>, ExternalMatchClientError> {
        self.assemble_malleable_quote_with_options(quote, AssembleQuoteOptions::default()).await
    }

    /// Assemble a quote into a malleable match bundle, ready for settlement,
    /// with options
    pub async fn assemble_malleable_quote_with_options(
        &self,
        quote: SignedExternalQuote,
        options: AssembleQuoteOptions,
    ) -> Result<Option<MalleableExternalMatchResponse>, ExternalMatchClientError> {
        let path = ASSEMBLE_EXTERNAL_MATCH_MALLEABLE_ROUTE;
        let signed_quote = ApiSignedQuote { quote: quote.quote, signature: quote.signature };
        let request = AssembleExternalMatchRequest {
            signed_quote,
            receiver_address: options.receiver_address.clone(),
            do_gas_estimation: options.do_gas_estimation,
            allow_shared: options.allow_shared,
            updated_order: options.updated_order.clone(),
        };
        let headers = self.get_headers()?;

        let resp = self.auth_http_client.post_with_headers_raw(path, request, headers).await?;
        let match_resp =
            Self::handle_optional_response::<MalleableExternalMatchResponse>(resp).await?;
        Ok(match_resp)
    }

    /// Request an external match
    #[deprecated(
        since = "0.1.0",
        note = "This endpoint will soon be removed, use `request_quote` and `assemble_quote` instead"
    )]
    #[allow(deprecated)]
    pub async fn request_external_match(
        &self,
        order: ExternalOrder,
    ) -> Result<Option<ExternalMatchResponse>, ExternalMatchClientError> {
        self.request_external_match_with_options(order, Default::default()).await
    }

    /// Request an external match and specify any options for the request
    #[deprecated(
        since = "0.1.0",
        note = "This endpoint will soon be removed, use `request_quote` and `assemble_quote` instead"
    )]
    #[allow(deprecated)]
    pub async fn request_external_match_with_options(
        &self,
        order: ExternalOrder,
        options: ExternalMatchOptions,
    ) -> Result<Option<ExternalMatchResponse>, ExternalMatchClientError> {
        let path = options.build_request_path();
        let do_gas_estimation = options.do_gas_estimation;
        let request = ExternalMatchRequest {
            external_order: order,
            do_gas_estimation,
            receiver_address: options.receiver_address,
        };
        let headers = self.get_headers()?;

        let resp =
            self.auth_http_client.post_with_headers_raw(path.as_str(), request, headers).await?;
        let match_resp = Self::handle_optional_response::<ExternalMatchResponse>(resp).await?;
        Ok(match_resp)
    }

    /// Helper function to handle response that might be NO_CONTENT, OK with
    /// json, or an error
    async fn handle_optional_response<T>(
        response: reqwest::Response,
    ) -> Result<Option<T>, ExternalMatchClientError>
    where
        T: serde::de::DeserializeOwned,
    {
        if response.status() == StatusCode::NO_CONTENT {
            Ok(None)
        } else if response.status() == StatusCode::OK {
            let resp = response.json::<T>().await?;
            Ok(Some(resp))
        } else {
            let status = response.status();
            let msg = response.text().await?;
            Err(ExternalMatchClientError::http(status, msg))
        }
    }

    /// Get a header map with the api key added
    fn get_headers(&self) -> Result<HeaderMap, ExternalMatchClientError> {
        let mut headers = HeaderMap::new();
        let api_key = HeaderValue::from_str(&self.api_key)
            .map_err(|_| ExternalMatchClientError::InvalidApiKey)?;
        headers.insert(RENEGADE_API_KEY_HEADER, api_key);

        Ok(headers)
    }
}
