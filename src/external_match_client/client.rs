//! The client for requesting external matches

use reqwest::{
    header::{HeaderMap, HeaderValue},
    StatusCode,
};

use crate::{
    api_types::{
        exchange_metadata::ExchangeMetadataResponse,
        order_book::{GetDepthByMintResponse, GetDepthForAllPairsResponse, PriceAndDepth},
        AssemblyType, ExternalMatchResponse, ASSEMBLE_MATCH_BUNDLE_ROUTE, ORDER_BOOK_DEPTH_ROUTE,
    },
    ARBITRUM_ONE_RELAYER_BASE_URL, ARBITRUM_SEPOLIA_RELAYER_BASE_URL,
    BASE_MAINNET_RELAYER_BASE_URL, BASE_SEPOLIA_RELAYER_BASE_URL,
};

use crate::{
    api_types::{GetTokenPricesResponse, GET_TOKEN_PRICES_ROUTE},
    http::RelayerHttpClient,
    util::HmacKey,
    AssembleQuoteOptions, ExternalMatchOptions, RequestQuoteOptions,
};

use super::{
    api_types::{
        ApiSignedQuote, AssembleExternalMatchRequest, ExternalOrder, ExternalQuoteRequest,
        ExternalQuoteResponse, GetSupportedTokensResponse, SignedExternalQuote,
        GET_EXCHANGE_METADATA_ROUTE, GET_SUPPORTED_TOKENS_ROUTE,
    },
    error::ExternalMatchClientError,
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

    // --------------------
    // | Orderbook Routes |
    // --------------------

    /// Get a list of supported tokens for external matches
    pub async fn get_supported_tokens(
        &self,
    ) -> Result<GetSupportedTokensResponse, ExternalMatchClientError> {
        let path = GET_SUPPORTED_TOKENS_ROUTE;
        let resp = self.relayer_http_client.get(path).await?;

        Ok(resp)
    }

    /// Get token prices for all supported tokens
    pub async fn get_token_prices(
        &self,
    ) -> Result<GetTokenPricesResponse, ExternalMatchClientError> {
        let path = GET_TOKEN_PRICES_ROUTE;
        let resp = self.relayer_http_client.get(path).await?;

        Ok(resp)
    }

    /// Get metadata about the Renegade exchange
    pub async fn get_exchange_metadata(
        &self,
    ) -> Result<ExchangeMetadataResponse, ExternalMatchClientError> {
        let path = GET_EXCHANGE_METADATA_ROUTE;
        let headers = self.get_headers()?;
        let resp = self.auth_http_client.get_with_headers(path, headers).await?;

        Ok(resp)
    }

    /// Get the order book depth for a token
    ///
    /// The address is the address of the token
    pub async fn get_order_book_depth(
        &self,
        address: &str,
    ) -> Result<GetDepthByMintResponse, ExternalMatchClientError> {
        let path = format!("{ORDER_BOOK_DEPTH_ROUTE}/{address}");
        let headers = self.get_headers()?;
        let resp: PriceAndDepth =
            self.auth_http_client.get_with_headers(path.as_str(), headers).await?;

        Ok(GetDepthByMintResponse { depth: resp })
    }

    /// Get the order book depth for all supported tokens
    pub async fn get_order_book_depth_all_pairs(
        &self,
    ) -> Result<GetDepthForAllPairsResponse, ExternalMatchClientError> {
        let path = ORDER_BOOK_DEPTH_ROUTE;
        let headers = self.get_headers()?;
        let resp = self.auth_http_client.get_with_headers(path, headers).await?;

        Ok(resp)
    }

    // -------------------------
    // | External Match Routes |
    // -------------------------

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
        Ok(quote_resp
            .map(|r| SignedExternalQuote::from_api_quote(r.signed_quote, r.gas_sponsorship_info)))
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
        let path = ASSEMBLE_MATCH_BUNDLE_ROUTE;

        let signed_quote = ApiSignedQuote::from(quote);
        let assembly =
            AssemblyType::QuotedOrder { signed_quote, updated_order: options.updated_order };

        let request = AssembleExternalMatchRequest {
            receiver_address: options.receiver_address,
            do_gas_estimation: options.do_gas_estimation,
            assembly,
        };

        let headers = self.get_headers()?;
        let resp = self.auth_http_client.post_with_headers_raw(path, request, headers).await?;

        let match_resp = Self::handle_optional_response::<ExternalMatchResponse>(resp).await?;
        Ok(match_resp)
    }

    /// Assemble a quote into a malleable match bundle, ready for settlement
    #[deprecated(
        since = "2.0.0",
        note = "Use assemble_quote instead, all matches are now malleable"
    )]
    pub async fn assemble_malleable_quote(
        &self,
        quote: SignedExternalQuote,
    ) -> Result<Option<ExternalMatchResponse>, ExternalMatchClientError> {
        self.assemble_quote(quote).await
    }

    /// Assemble a quote into a malleable match bundle, ready for settlement,
    /// with options
    #[deprecated(
        since = "2.0.0",
        note = "Use assemble_quote_with_options instead, all matches are now malleable"
    )]
    pub async fn assemble_malleable_quote_with_options(
        &self,
        quote: SignedExternalQuote,
        options: AssembleQuoteOptions,
    ) -> Result<Option<ExternalMatchResponse>, ExternalMatchClientError> {
        self.assemble_quote_with_options(quote, options).await
    }

    /// Request an external match
    pub async fn request_external_match(
        &self,
        order: ExternalOrder,
    ) -> Result<Option<ExternalMatchResponse>, ExternalMatchClientError> {
        self.request_external_match_with_options(order, Default::default()).await
    }

    /// Request an external match and specify any options for the request
    pub async fn request_external_match_with_options(
        &self,
        order: ExternalOrder,
        options: ExternalMatchOptions,
    ) -> Result<Option<ExternalMatchResponse>, ExternalMatchClientError> {
        let path = options.build_request_path();

        let assembly = AssemblyType::NewOrder { external_order: order };

        let request = AssembleExternalMatchRequest {
            receiver_address: options.receiver_address,
            do_gas_estimation: options.do_gas_estimation,
            assembly,
        };

        let headers = self.get_headers()?;
        let resp =
            self.auth_http_client.post_with_headers_raw(path.as_str(), request, headers).await?;

        let match_resp = Self::handle_optional_response::<ExternalMatchResponse>(resp).await?;
        Ok(match_resp)
    }

    /// Request a malleable external match
    #[deprecated(
        since = "2.0.0",
        note = "Use request_external_match instead, all matches are now malleable"
    )]
    pub async fn request_malleable_external_match(
        &self,
        order: ExternalOrder,
    ) -> Result<Option<ExternalMatchResponse>, ExternalMatchClientError> {
        self.request_external_match(order).await
    }

    /// Request a malleable external match and specify any options for the
    /// request
    #[deprecated(
        since = "2.0.0",
        note = "Use request_external_match_with_options instead, all matches are now malleable"
    )]
    pub async fn request_malleable_external_match_with_options(
        &self,
        order: ExternalOrder,
        options: ExternalMatchOptions,
    ) -> Result<Option<ExternalMatchResponse>, ExternalMatchClientError> {
        self.request_external_match_with_options(order, options).await
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
