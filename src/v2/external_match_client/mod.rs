//! A client for requesting external matches from the relayer.
//!
//! An external match is one between an internal party -- one with state
//! committed into the Renegade darkpool, and an external party -- one with no
//! state committed into the Renegade darkpool.

use crate::{
    v2::{
        ARBITRUM_ONE_RELAYER_BASE_URL, ARBITRUM_SEPOLIA_RELAYER_BASE_URL,
        BASE_MAINNET_RELAYER_BASE_URL, BASE_SEPOLIA_RELAYER_BASE_URL,
    },
    HmacKey, RelayerHttpClient,
};

mod error;
pub use error::ExternalMatchClientError;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    StatusCode,
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

// ----------
// | Client |
// ----------

/// A client for requesting external matches from the relayer
#[derive(Clone, Debug)]
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

    /// Create a new client with a custom HTTP client
    pub fn new_with_client(
        api_key: &str,
        api_secret: &str,
        auth_base_url: &str,
        relayer_base_url: &str,
        client: reqwest::Client,
    ) -> Result<Self, ExternalMatchClientError> {
        let api_secret = HmacKey::from_base64_string(api_secret)
            .map_err(|_| ExternalMatchClientError::InvalidApiSecret)?;

        let auth_http_client = RelayerHttpClient::new_with_client(
            auth_base_url.to_string(),
            api_secret,
            client.clone(),
        );
        let relayer_http_client =
            RelayerHttpClient::new_with_client(relayer_base_url.to_string(), api_secret, client);

        Ok(Self { api_key: api_key.to_string(), auth_http_client, relayer_http_client })
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

    /// Create a new client for the Arbitrum One network
    pub fn new_arbitrum_one_client(
        api_key: &str,
        api_secret: &str,
    ) -> Result<Self, ExternalMatchClientError> {
        Self::new(api_key, api_secret, ARBITRUM_ONE_AUTH_BASE_URL, ARBITRUM_ONE_RELAYER_BASE_URL)
    }

    /// Create a new client for the Arbitrum One network with custom HTTP client
    pub fn new_arbitrum_one_with_client(
        api_key: &str,
        api_secret: &str,
        client: reqwest::Client,
    ) -> Result<Self, ExternalMatchClientError> {
        Self::new_with_client(
            api_key,
            api_secret,
            ARBITRUM_ONE_AUTH_BASE_URL,
            ARBITRUM_ONE_RELAYER_BASE_URL,
            client,
        )
    }

    /// Create a new client for the Base mainnet network
    pub fn new_base_mainnet_client(
        api_key: &str,
        api_secret: &str,
    ) -> Result<Self, ExternalMatchClientError> {
        Self::new(api_key, api_secret, BASE_MAINNET_AUTH_BASE_URL, BASE_MAINNET_RELAYER_BASE_URL)
    }

    /// Create a new client for the Base mainnet network with custom HTTP client
    pub fn new_base_mainnet_with_client(
        api_key: &str,
        api_secret: &str,
        client: reqwest::Client,
    ) -> Result<Self, ExternalMatchClientError> {
        Self::new_with_client(
            api_key,
            api_secret,
            BASE_MAINNET_AUTH_BASE_URL,
            BASE_MAINNET_RELAYER_BASE_URL,
            client,
        )
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
