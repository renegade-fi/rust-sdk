//! The client for requesting external matches

use renegade_types_core::HmacKey;
use reqwest::{
    StatusCode,
    header::{HeaderMap, HeaderValue},
};

use crate::{
    ARBITRUM_ONE_RELAYER_BASE_URL, ARBITRUM_SEPOLIA_RELAYER_BASE_URL, AssembleQuoteOptionsV2,
    BASE_MAINNET_RELAYER_BASE_URL, BASE_SEPOLIA_RELAYER_BASE_URL,
    ETHEREUM_SEPOLIA_RELAYER_BASE_URL, ExternalMatchOptions, RequestQuoteOptions,
    api_types::{
        ASSEMBLE_MATCH_BUNDLE_ROUTE, AssemblyType, ExternalMatchResponseV2,
        GET_MARKET_DEPTH_BY_MINT_ROUTE, GET_MARKETS_DEPTH_ROUTE, GET_MARKETS_ROUTE,
        GetMarketDepthByMintResponse, GetMarketDepthsResponse, GetMarketsResponse,
        exchange_metadata::ExchangeMetadataResponse,
    },
};

#[allow(deprecated)]
use crate::http::RelayerHttpClient;

use super::{
    api_types::{
        ApiSignedQuoteV2, AssembleExternalMatchRequest, ExternalOrderV2, ExternalQuoteRequest,
        ExternalQuoteResponse, GET_EXCHANGE_METADATA_ROUTE, SignedExternalQuoteV2,
    },
    error::ExternalMatchClientError,
};

// -------------
// | Constants |
// -------------

/// The Renegade API key header
pub const RENEGADE_API_KEY_HEADER: &str = "X-Renegade-Api-Key";

/// The Arbitrum Sepolia auth server base URL
const ARBITRUM_SEPOLIA_AUTH_BASE_URL: &str = "https://arbitrum-sepolia.v2.auth-server.renegade.fi";
/// The Arbitrum One auth server base URL
const ARBITRUM_ONE_AUTH_BASE_URL: &str = "https://arbitrum-one.v2.auth-server.renegade.fi";
/// The Base Sepolia auth server base URL
const BASE_SEPOLIA_AUTH_BASE_URL: &str = "https://base-sepolia.v2.auth-server.renegade.fi";
/// The Base mainnet auth server base URL
const BASE_MAINNET_AUTH_BASE_URL: &str = "https://base-mainnet.v2.auth-server.renegade.fi";
/// The Ethereum Sepolia auth server base URL
const ETHEREUM_SEPOLIA_AUTH_BASE_URL: &str = "https://ethereum-sepolia.v2.auth-server.renegade.fi";

// ----------
// | Client |
// ----------

/// A client for requesting external matches from the relayer
#[derive(Clone, Debug)]
pub struct ExternalMatchClient {
    /// The api key for the external match client
    pub(crate) api_key: String,
    /// The HTTP client
    pub(crate) auth_http_client: RelayerHttpClient,
    /// The relayer HTTP client
    ///
    /// Separate from the auth client as they request different base URLs
    pub(crate) relayer_http_client: RelayerHttpClient,
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

    /// Create a new client for the Ethereum Sepolia network
    pub fn new_ethereum_sepolia_client(
        api_key: &str,
        api_secret: &str,
    ) -> Result<Self, ExternalMatchClientError> {
        Self::new(
            api_key,
            api_secret,
            ETHEREUM_SEPOLIA_AUTH_BASE_URL,
            ETHEREUM_SEPOLIA_RELAYER_BASE_URL,
        )
    }

    /// Create a new client for the Ethereum Sepolia network with custom HTTP
    /// client
    pub fn new_ethereum_sepolia_with_client(
        api_key: &str,
        api_secret: &str,
        client: reqwest::Client,
    ) -> Result<Self, ExternalMatchClientError> {
        Self::new_with_client(
            api_key,
            api_secret,
            ETHEREUM_SEPOLIA_AUTH_BASE_URL,
            ETHEREUM_SEPOLIA_RELAYER_BASE_URL,
            client,
        )
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

    // ------------------
    // | Markets Routes |
    // ------------------

    /// Get a list of tradable markets. Includes the tokens pair, current price,
    /// and fee rates for each market.
    pub async fn get_markets(&self) -> Result<GetMarketsResponse, ExternalMatchClientError> {
        let path = GET_MARKETS_ROUTE;
        let resp = self.relayer_http_client.get(path).await?;

        Ok(resp)
    }

    /// Get the market depth for the given token.
    ///
    /// The address is the address of the token
    pub async fn get_market_depth(
        &self,
        address: &str,
    ) -> Result<GetMarketDepthByMintResponse, ExternalMatchClientError> {
        let path = GET_MARKET_DEPTH_BY_MINT_ROUTE.replace(":mint", address);
        let headers = self.get_headers()?;
        let resp = self.auth_http_client.get_with_headers(&path, headers).await?;

        Ok(resp)
    }

    /// Get the market depths for all supported pairs
    pub async fn get_market_depths_all_pairs(
        &self,
    ) -> Result<GetMarketDepthsResponse, ExternalMatchClientError> {
        let path = GET_MARKETS_DEPTH_ROUTE;
        let headers = self.get_headers()?;
        let resp = self.auth_http_client.get_with_headers(path, headers).await?;

        Ok(resp)
    }

    // -------------------------
    // | External Match Routes |
    // -------------------------

    /// Request a quote for an external match (v2 API)
    pub async fn request_quote_v2(
        &self,
        order: ExternalOrderV2,
    ) -> Result<Option<SignedExternalQuoteV2>, ExternalMatchClientError> {
        self.request_quote_with_options_v2(order, RequestQuoteOptions::default()).await
    }

    /// Request a quote for an external match, with options (v2 API)
    pub async fn request_quote_with_options_v2(
        &self,
        order: ExternalOrderV2,
        options: RequestQuoteOptions,
    ) -> Result<Option<SignedExternalQuoteV2>, ExternalMatchClientError> {
        let request = ExternalQuoteRequest { external_order: order };
        let path = options.build_request_path();
        let headers = self.get_headers()?;

        let resp = self.auth_http_client.post_with_headers_raw(&path, request, headers).await?;
        let quote_resp = Self::handle_optional_response::<ExternalQuoteResponse>(resp).await?;
        Ok(quote_resp
            .map(|r| SignedExternalQuoteV2::from_api_quote(r.signed_quote, r.gas_sponsorship_info)))
    }

    /// Assemble a quote into a match bundle, ready for settlement (v2 API)
    pub async fn assemble_quote_v2(
        &self,
        quote: SignedExternalQuoteV2,
    ) -> Result<Option<ExternalMatchResponseV2>, ExternalMatchClientError> {
        self.assemble_quote_with_options_v2(quote, AssembleQuoteOptionsV2::default()).await
    }

    /// Assemble a quote into a match bundle, ready for settlement, with options
    /// (v2 API)
    pub async fn assemble_quote_with_options_v2(
        &self,
        quote: SignedExternalQuoteV2,
        options: AssembleQuoteOptionsV2,
    ) -> Result<Option<ExternalMatchResponseV2>, ExternalMatchClientError> {
        let path = ASSEMBLE_MATCH_BUNDLE_ROUTE;

        let signed_quote = ApiSignedQuoteV2::from(quote);
        let order =
            AssemblyType::QuotedOrder { signed_quote, updated_order: options.updated_order };

        let request = AssembleExternalMatchRequest {
            receiver_address: options.receiver_address,
            do_gas_estimation: options.do_gas_estimation,
            order,
        };

        let headers = self.get_headers()?;
        let resp = self.auth_http_client.post_with_headers_raw(path, request, headers).await?;

        let match_resp = Self::handle_optional_response::<ExternalMatchResponseV2>(resp).await?;
        Ok(match_resp)
    }

    /// Request an external match (v2 API)
    pub async fn request_external_match_v2(
        &self,
        order: ExternalOrderV2,
    ) -> Result<Option<ExternalMatchResponseV2>, ExternalMatchClientError> {
        self.request_external_match_with_options_v2(order, Default::default()).await
    }

    /// Request an external match and specify any options for the request (v2
    /// API)
    pub async fn request_external_match_with_options_v2(
        &self,
        order: ExternalOrderV2,
        options: ExternalMatchOptions,
    ) -> Result<Option<ExternalMatchResponseV2>, ExternalMatchClientError> {
        let path = options.build_request_path();

        let order = AssemblyType::DirectOrder { external_order: order };
        let request = AssembleExternalMatchRequest {
            receiver_address: options.receiver_address,
            do_gas_estimation: options.do_gas_estimation,
            order,
        };

        let headers = self.get_headers()?;
        let resp =
            self.auth_http_client.post_with_headers_raw(path.as_str(), request, headers).await?;

        let match_resp = Self::handle_optional_response::<ExternalMatchResponseV2>(resp).await?;
        Ok(match_resp)
    }

    // -------------------
    // | Metadata Routes |
    // -------------------

    /// Get metadata about the Renegade exchange
    pub async fn get_exchange_metadata(
        &self,
    ) -> Result<ExchangeMetadataResponse, ExternalMatchClientError> {
        let path = GET_EXCHANGE_METADATA_ROUTE;
        let headers = self.get_headers()?;
        let resp = self.auth_http_client.get_with_headers(path, headers).await?;

        Ok(resp)
    }

    // -----------
    // | Helpers |
    // -----------

    /// Helper function to handle response that might be NO_CONTENT, OK with
    /// json, or an error
    pub(crate) async fn handle_optional_response<T>(
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
    pub(crate) fn get_headers(&self) -> Result<HeaderMap, ExternalMatchClientError> {
        let mut headers = HeaderMap::new();
        let api_key = HeaderValue::from_str(&self.api_key)
            .map_err(|_| ExternalMatchClientError::InvalidApiKey)?;
        headers.insert(RENEGADE_API_KEY_HEADER, api_key);

        Ok(headers)
    }
}
