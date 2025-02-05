//! The client for requesting external matches

use renegade_api::http::external_match::{
    AssembleExternalMatchRequest, AtomicMatchApiBundle, ExternalMatchRequest,
    ExternalMatchResponse, ExternalOrder, ExternalQuoteRequest, ExternalQuoteResponse,
    SignedExternalQuote, ASSEMBLE_EXTERNAL_MATCH_ROUTE, REQUEST_EXTERNAL_MATCH_ROUTE,
    REQUEST_EXTERNAL_QUOTE_ROUTE,
};
use renegade_auth_api::RENEGADE_API_KEY_HEADER;
use renegade_common::types::hmac::HmacKey;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    StatusCode,
};
use url::form_urlencoded;

use crate::http::RelayerHttpClient;

use super::{
    error::ExternalMatchClientError, GAS_REFUND_ADDRESS_QUERY_PARAM, GAS_SPONSORSHIP_QUERY_PARAM,
};

/// The sepolia base URL
const SEPOLIA_BASE_URL: &str = "https://testnet.auth-server.renegade.fi";
/// The mainnet base URL
const MAINNET_BASE_URL: &str = "https://mainnet.auth-server.renegade.fi";

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
        query.append_pair(GAS_SPONSORSHIP_QUERY_PARAM, &self.sponsor_gas.to_string());
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
    pub(crate) fn build_request_path(&self) -> String {
        let mut query = form_urlencoded::Serializer::new(String::new());
        query.append_pair(GAS_SPONSORSHIP_QUERY_PARAM, &self.sponsor_gas.to_string());

        if let Some(addr) = &self.gas_refund_address {
            query.append_pair(GAS_REFUND_ADDRESS_QUERY_PARAM, addr);
        }

        format!("{}?{}", ASSEMBLE_EXTERNAL_MATCH_ROUTE, query.finish())
    }
}

/// A client for requesting external matches from the relayer
#[derive(Clone)]
pub struct ExternalMatchClient {
    /// The api key for the external match client
    api_key: String,
    /// The HTTP client
    http_client: RelayerHttpClient,
}

impl ExternalMatchClient {
    /// Create a new client
    pub fn new(
        api_key: &str,
        api_secret: &str,
        base_url: &str,
    ) -> Result<Self, ExternalMatchClientError> {
        let api_secret = HmacKey::from_base64_string(api_secret)
            .map_err(|_| ExternalMatchClientError::InvalidApiSecret)?;

        Ok(Self {
            api_key: api_key.to_string(),
            http_client: RelayerHttpClient::new(base_url.to_string(), api_secret),
        })
    }

    /// Create a new client for the sepolia network
    pub fn new_sepolia_client(
        api_key: &str,
        api_secret: &str,
    ) -> Result<Self, ExternalMatchClientError> {
        Self::new(api_key, api_secret, SEPOLIA_BASE_URL)
    }

    /// Create a new client for the mainnet
    pub fn new_mainnet_client(
        api_key: &str,
        api_secret: &str,
    ) -> Result<Self, ExternalMatchClientError> {
        Self::new(api_key, api_secret, MAINNET_BASE_URL)
    }

    /// Request a quote for an external match
    pub async fn request_quote(
        &self,
        order: ExternalOrder,
    ) -> Result<Option<SignedExternalQuote>, ExternalMatchClientError> {
        let request = ExternalQuoteRequest { external_order: order };
        let path = REQUEST_EXTERNAL_QUOTE_ROUTE;
        let headers = self.get_headers()?;

        let resp = self.http_client.post_with_headers_raw(path, request, headers).await?;
        let quote_resp = Self::handle_optional_response::<ExternalQuoteResponse>(resp).await?;
        Ok(quote_resp.map(|r| r.signed_quote))
    }

    /// Assemble a quote into a match bundle, ready for settlement
    pub async fn assemble_quote(
        &self,
        quote: SignedExternalQuote,
    ) -> Result<Option<AtomicMatchApiBundle>, ExternalMatchClientError> {
        self.assemble_quote_with_options(quote, AssembleQuoteOptions::default()).await
    }

    /// Assemble a quote into a match bundle, ready for settlement, with options
    pub async fn assemble_quote_with_options(
        &self,
        quote: SignedExternalQuote,
        options: AssembleQuoteOptions,
    ) -> Result<Option<AtomicMatchApiBundle>, ExternalMatchClientError> {
        let path = options.build_request_path();
        let request = AssembleExternalMatchRequest {
            signed_quote: quote,
            receiver_address: options.receiver_address,
            do_gas_estimation: options.do_gas_estimation,
            updated_order: options.updated_order,
        };
        let headers = self.get_headers()?;

        let resp = self.http_client.post_with_headers_raw(path.as_str(), request, headers).await?;
        let match_resp = Self::handle_optional_response::<ExternalMatchResponse>(resp).await?;
        Ok(match_resp.map(|r| r.match_bundle))
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
    ) -> Result<Option<AtomicMatchApiBundle>, ExternalMatchClientError> {
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
    ) -> Result<Option<AtomicMatchApiBundle>, ExternalMatchClientError> {
        let path = options.build_request_path();
        let do_gas_estimation = options.do_gas_estimation;
        let request = ExternalMatchRequest {
            external_order: order,
            do_gas_estimation,
            receiver_address: options.receiver_address,
        };
        let headers = self.get_headers()?;

        let resp = self.http_client.post_with_headers_raw(path.as_str(), request, headers).await?;
        let match_resp = Self::handle_optional_response::<ExternalMatchResponse>(resp).await?;
        Ok(match_resp.map(|r| r.match_bundle))
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
