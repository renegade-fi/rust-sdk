//! HTTP client for connecting to the relayer

use renegade_common::types::wallet::keychain::HmacKey;
use reqwest::{header::HeaderMap, Client, Error};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

/// The duration for which request signatures are valid
const REQUEST_SIGNATURE_DURATION: Duration = Duration::from_secs(10);

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
        self.add_auth(path, &mut custom_headers, &body_bytes);

        let raw = self.client.post(url).headers(custom_headers).body(body_bytes).send().await?;
        let response = raw.error_for_status()?;
        Ok(response)
    }

    /// Send a GET request with custom headers to the relayer and return raw
    /// response
    pub async fn get_with_headers_raw(
        &self,
        path: &str,
        mut custom_headers: HeaderMap,
    ) -> Result<reqwest::Response, Error> {
        let url = format!("{}{}", self.base_url, path);
        self.add_auth(path, &mut custom_headers, &[]);

        let raw = self.client.get(url).headers(custom_headers).send().await?;
        let response = raw.error_for_status()?;
        Ok(response)
    }

    // -----------
    // | Helpers |
    // -----------

    /// Add authentication to the request
    fn add_auth(&self, path: &str, headers: &mut HeaderMap, body: &[u8]) {
        renegade_api::auth::add_expiring_auth_to_headers(
            path,
            headers,
            body,
            &self.auth_key,
            REQUEST_SIGNATURE_DURATION,
        );
    }
}