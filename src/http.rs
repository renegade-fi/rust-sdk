//! HTTP client for connecting to the relayer

use renegade_external_api::auth::add_expiring_auth_to_headers;
use renegade_types_core::HmacKey;
use reqwest::{header::HeaderMap, Client};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use url::Url;

/// The duration for which request signatures are valid
const REQUEST_SIGNATURE_DURATION: Duration = Duration::from_secs(10);

/// The header name for the SDK version
const SDK_VERSION_HEADER: &str = "x-renegade-sdk-version";

/// The error message when a response body cannot be decoded
const RESPONSE_BODY_DECODE_ERROR: &str = "<failed to decode response body>";

/// The error type for the relayer HTTP client
#[derive(Debug, thiserror::Error)]
pub enum RelayerHttpClientError {
    /// An error making an HTTP request
    #[error("HTTP error: {0}")]
    Http(reqwest::Error),
    /// An error in de/serialization
    #[error("serde error: {0}")]
    Serde(String),
    /// An error parsing a value
    #[error("parse error: {0}")]
    Parse(String),
}

impl RelayerHttpClientError {
    /// Create a new `Parse` error
    #[allow(clippy::needless_pass_by_value)]
    pub fn parse<T: ToString>(e: T) -> Self {
        Self::Parse(e.to_string())
    }
}

impl From<reqwest::Error> for RelayerHttpClientError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err)
    }
}

/// An HTTP client for connecting to the relayer
#[derive(Clone, Debug)]
pub struct RelayerHttpClient {
    /// The HTTP client
    client: Client,
    /// The base URL of the relayer
    base_url: String,
    /// The authentication key to use for requests
    auth_key: HmacKey,
}

#[allow(unused)]
impl RelayerHttpClient {
    /// Create a new HTTP client
    pub fn new(base_url: String, auth_key: HmacKey) -> Self {
        Self { client: Client::new(), base_url, auth_key }
    }
    /// Create a new HTTP client
    pub fn new_with_client(base_url: String, auth_key: HmacKey, client: reqwest::Client) -> Self {
        Self { client, base_url, auth_key }
    }

    /// Send a POST request to the relayer
    pub async fn post<Req: Serialize, Resp: DeserializeOwned>(
        &self,
        path: &str,
        body: Req,
    ) -> Result<Resp, RelayerHttpClientError> {
        self.post_with_headers(path, body, HeaderMap::new()).await
    }

    /// Send a GET request to the relayer
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, RelayerHttpClientError> {
        self.get_with_headers(path, HeaderMap::new()).await
    }

    /// Send a POST request with custom headers to the relayer
    pub async fn post_with_headers<Req: Serialize, Resp: DeserializeOwned>(
        &self,
        path: &str,
        body: Req,
        custom_headers: HeaderMap,
    ) -> Result<Resp, RelayerHttpClientError> {
        let response = self.post_with_headers_raw(path, body, custom_headers).await?;
        let body = response.text().await.unwrap_or_else(|_| RESPONSE_BODY_DECODE_ERROR.to_string());

        // Attempt to decode the response body as the expected type
        // Otherwise, emit the body as an error
        let decoded: Result<Resp, _> = serde_json::from_str(&body);
        if let Ok(decoded) = decoded {
            Ok(decoded)
        } else {
            Err(RelayerHttpClientError::Serde(body))
        }
    }

    /// Send a GET request with custom headers to the relayer
    pub async fn get_with_headers<T: DeserializeOwned>(
        &self,
        path: &str,
        custom_headers: HeaderMap,
    ) -> Result<T, RelayerHttpClientError> {
        let response = self.get_with_headers_raw(path, custom_headers).await?;
        let body = response.text().await.unwrap_or_else(|_| RESPONSE_BODY_DECODE_ERROR.to_string());

        // Attempt to decode the response body as the expected type
        // Otherwise, emit the body as an error
        let decoded: Result<T, _> = serde_json::from_str(&body);
        if let Ok(decoded) = decoded {
            Ok(decoded)
        } else {
            Err(RelayerHttpClientError::Serde(body))
        }
    }

    /// Send a POST request with custom headers to the relayer and return raw
    /// response
    pub async fn post_with_headers_raw<Req: Serialize>(
        &self,
        path: &str,
        body: Req,
        mut custom_headers: HeaderMap,
    ) -> Result<reqwest::Response, RelayerHttpClientError> {
        let url_raw = format!("{}{}", self.base_url, path);
        let url = Url::parse(&url_raw).map_err(RelayerHttpClientError::parse)?;

        let body_bytes = serde_json::to_vec(&body).unwrap();
        self.add_headers(&url, &mut custom_headers, &body_bytes);

        let raw = self.client.post(url).headers(custom_headers).body(body_bytes).send().await?;
        Ok(raw)
    }

    /// Send a GET request with custom headers to the relayer and return raw
    /// response
    pub async fn get_with_headers_raw(
        &self,
        path: &str,
        mut custom_headers: HeaderMap,
    ) -> Result<reqwest::Response, RelayerHttpClientError> {
        let url_raw = format!("{}{}", self.base_url, path);
        let url = Url::parse(&url_raw).map_err(RelayerHttpClientError::parse)?;

        self.add_headers(&url, &mut custom_headers, &[]);

        let raw = self.client.get(url).headers(custom_headers).send().await?;
        Ok(raw)
    }

    // -----------
    // | Helpers |
    // -----------

    /// Get the SDK version
    fn get_sdk_version() -> String {
        let version_string = env!("CARGO_PKG_VERSION");
        format!("rust-v{version_string}")
    }

    /// Add authentication and SDK version headers to the request
    fn add_headers(&self, url: &Url, headers: &mut HeaderMap, body: &[u8]) {
        let path = url.path().to_string();
        let query = url.query().map(|q| format!("?{q}")).unwrap_or_default();
        let full_path = format!("{path}{query}");

        // Add SDK version header
        let sdk_version = Self::get_sdk_version();
        headers.insert(SDK_VERSION_HEADER, sdk_version.parse().unwrap());
        add_expiring_auth_to_headers(
            &full_path,
            headers,
            body,
            &self.auth_key,
            REQUEST_SIGNATURE_DURATION,
        );
    }
}
