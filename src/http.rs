//! HTTP client for connecting to the relayer

use crate::util::{self, HmacKey};

use reqwest::{header::HeaderMap, Client, Error};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

/// The duration for which request signatures are valid
const REQUEST_SIGNATURE_DURATION: Duration = Duration::from_secs(10);

/// The header name for the SDK version
const SDK_VERSION_HEADER: &str = "x-renegade-sdk-version";

/// An HTTP client for connecting to the relayer
#[derive(Clone)]
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
        Self { client: client, base_url, auth_key }
    }

    /// Send a POST request to the relayer
    pub async fn post<Req: Serialize, Resp: DeserializeOwned>(
        &self,
        path: &str,
        body: Req,
    ) -> Result<Resp, Error> {
        self.post_with_headers(path, body, HeaderMap::new()).await
    }

    /// Send a GET request to the relayer
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, Error> {
        self.get_with_headers(path, HeaderMap::new()).await
    }

    /// Send a POST request with custom headers to the relayer
    pub async fn post_with_headers<Req: Serialize, Resp: DeserializeOwned>(
        &self,
        path: &str,
        body: Req,
        custom_headers: HeaderMap,
    ) -> Result<Resp, Error> {
        let response = self.post_with_headers_raw(path, body, custom_headers).await?;
        response.json().await
    }

    /// Send a GET request with custom headers to the relayer
    pub async fn get_with_headers<T: DeserializeOwned>(
        &self,
        path: &str,
        custom_headers: HeaderMap,
    ) -> Result<T, Error> {
        let response = self.get_with_headers_raw(path, custom_headers).await?;
        response.json().await
    }

    /// Send a POST request with custom headers to the relayer and return raw
    /// response
    pub async fn post_with_headers_raw<Req: Serialize>(
        &self,
        path: &str,
        body: Req,
        mut custom_headers: HeaderMap,
    ) -> Result<reqwest::Response, Error> {
        let url = format!("{}{}", self.base_url, path);
        let body_bytes = serde_json::to_vec(&body).unwrap();
        self.add_headers(path, &mut custom_headers, &body_bytes);

        let raw = self.client.post(url).headers(custom_headers).body(body_bytes).send().await?;
        Ok(raw)
    }

    /// Send a GET request with custom headers to the relayer and return raw
    /// response
    pub async fn get_with_headers_raw(
        &self,
        path: &str,
        mut custom_headers: HeaderMap,
    ) -> Result<reqwest::Response, Error> {
        let url = format!("{}{}", self.base_url, path);
        self.add_headers(path, &mut custom_headers, &[]);

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
    fn add_headers(&self, path: &str, headers: &mut HeaderMap, body: &[u8]) {
        // Add SDK version header
        let sdk_version = Self::get_sdk_version();
        headers.insert(SDK_VERSION_HEADER, sdk_version.parse().unwrap());
        util::add_expiring_auth_to_headers(
            path,
            headers,
            body,
            &self.auth_key,
            REQUEST_SIGNATURE_DURATION,
        );
    }
}
