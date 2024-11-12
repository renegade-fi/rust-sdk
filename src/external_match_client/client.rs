//! The client for requesting external matches

use renegade_api::http::external_match::{
    AtomicMatchApiBundle, ExternalMatchRequest, ExternalMatchResponse, ExternalOrder,
    REQUEST_EXTERNAL_MATCH_ROUTE,
};
use renegade_auth_api::RENEGADE_API_KEY_HEADER;
use renegade_common::types::wallet::keychain::HmacKey;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    StatusCode,
};

use crate::http::RelayerHttpClient;

use super::error::ExternalMatchClientError;

/// The sepolia base URL
const SEPOLIA_BASE_URL: &str = "https://testnet.auth-server.renegade.fi";
/// The mainnet base URL
const MAINNET_BASE_URL: &str = "https://mainnet.auth-server.renegade.fi";

/// The options for requesting an external match
#[derive(Clone, Default)]
pub struct ExternalMatchOptions {
    /// Whether to perform gas estimation
    pub do_gas_estimation: bool,
}

impl ExternalMatchOptions {
    /// Create a new options with default values
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a new options with gas estimation enabled
    pub fn with_gas_estimation(mut self, do_gas_estimation: bool) -> Self {
        self.do_gas_estimation = do_gas_estimation;
        self
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

    /// Request an external match
    pub async fn request_external_match(
        &self,
        order: ExternalOrder,
    ) -> Result<Option<AtomicMatchApiBundle>, ExternalMatchClientError> {
        self.request_external_match_with_options(order, Default::default()).await
    }

    /// Request an external match and specify any options for the request
    pub async fn request_external_match_with_options(
        &self,
        order: ExternalOrder,
        options: ExternalMatchOptions,
    ) -> Result<Option<AtomicMatchApiBundle>, ExternalMatchClientError> {
        // Build the request, we attach the api key as a header and let the auth path
        // sign with the api secret
        let do_gas_estimation = options.do_gas_estimation;
        let request = ExternalMatchRequest { external_order: order, do_gas_estimation };

        let mut headers = HeaderMap::new();
        let api_key = HeaderValue::from_str(&self.api_key)
            .map_err(|_| ExternalMatchClientError::InvalidApiKey)?;
        headers.insert(RENEGADE_API_KEY_HEADER, api_key);

        // Send the request and handle the response
        let path = REQUEST_EXTERNAL_MATCH_ROUTE;
        let resp = self.http_client.post_with_headers_raw(path, request, headers).await?;

        if resp.status() == StatusCode::NO_CONTENT {
            Ok(None)
        } else if resp.status() == StatusCode::OK {
            let resp = resp.json::<ExternalMatchResponse>().await?;
            Ok(Some(resp.match_bundle))
        } else {
            let status = resp.status();
            let msg = resp.text().await?;
            Err(ExternalMatchClientError::http(status, msg))
        }
    }
}
