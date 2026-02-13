//! v1-compatible client methods on `ExternalMatchClient`
//!
//! These methods accept v1 types and shim them through to the v2 API under the
//! hood.

use crate::{
    ExternalMatchOptions, RequestQuoteOptions,
    api_types::v1_types::{
        ExternalMatchResponse, ExternalOrder, GetDepthByMintResponse, GetDepthForAllPairsResponse,
        GetSupportedTokensResponse, GetTokenPricesResponse, SignedExternalQuote,
    },
};

use super::{
    ExternalMatchClient, ExternalMatchClientError,
    options::AssembleQuoteOptions,
    v1_conversions::{
        market_depth_to_v1, market_depths_to_v1, markets_to_supported_tokens,
        markets_to_token_prices, v1_assemble_options_to_v2, v1_order_to_v2, v1_quote_to_v2,
        v2_quote_to_v1, v2_response_to_v1_non_malleable,
    },
};

impl ExternalMatchClient {
    // -------------------------
    // | Market / Token Routes |
    // -------------------------

    /// Get a list of supported tokens for external matches
    #[deprecated(
        since = "2.0.0",
        note = "Use get_markets instead, which returns all supported tokens along with their current price"
    )]
    pub async fn get_supported_tokens(
        &self,
    ) -> Result<GetSupportedTokensResponse, ExternalMatchClientError> {
        let resp = self.get_markets().await?;
        Ok(markets_to_supported_tokens(&resp))
    }

    /// Get token prices for all supported tokens
    #[deprecated(
        since = "2.0.0",
        note = "Use get_markets instead, which returns all supported tokens along with their current price"
    )]
    pub async fn get_token_prices(
        &self,
    ) -> Result<GetTokenPricesResponse, ExternalMatchClientError> {
        let resp = self.get_markets().await?;
        markets_to_token_prices(&resp)
    }

    /// Get the order book depth for a token
    ///
    /// The address is the address of the token
    #[deprecated(since = "2.0.0", note = "Use get_market_depth instead")]
    pub async fn get_order_book_depth(
        &self,
        address: &str,
    ) -> Result<GetDepthByMintResponse, ExternalMatchClientError> {
        let resp = self.get_market_depth(address).await?;
        market_depth_to_v1(&resp)
    }

    /// Get the order book depth for all supported tokens
    #[deprecated(since = "2.0.0", note = "Use get_market_depths_all_pairs instead")]
    pub async fn get_order_book_depth_all_pairs(
        &self,
    ) -> Result<GetDepthForAllPairsResponse, ExternalMatchClientError> {
        let resp = self.get_market_depths_all_pairs().await?;
        market_depths_to_v1(&resp)
    }

    // ---------------------
    // | Quote Routes (v1) |
    // ---------------------

    /// Request a quote for an external match (v1 API)
    pub async fn request_quote(
        &self,
        order: ExternalOrder,
    ) -> Result<Option<SignedExternalQuote>, ExternalMatchClientError> {
        self.request_quote_with_options(order, RequestQuoteOptions::default()).await
    }

    /// Request a quote for an external match, with options (v1 API)
    pub async fn request_quote_with_options(
        &self,
        order: ExternalOrder,
        options: RequestQuoteOptions,
    ) -> Result<Option<SignedExternalQuote>, ExternalMatchClientError> {
        let v2_order = v1_order_to_v2(&order);
        let v2_quote = self.request_quote_with_options_v2(v2_order, options).await?;
        v2_quote.map(|q| v2_quote_to_v1(q, &order)).transpose()
    }

    /// Assemble a quote into a match bundle, ready for settlement (v1 API)
    ///
    /// Returns a non-malleable `ExternalMatchResponse`
    pub async fn assemble_quote(
        &self,
        quote: SignedExternalQuote,
    ) -> Result<Option<ExternalMatchResponse>, ExternalMatchClientError> {
        self.assemble_quote_with_options(quote, AssembleQuoteOptions::default()).await
    }

    /// Assemble a quote into a match bundle with options (v1 API)
    ///
    /// Returns a non-malleable `ExternalMatchResponse`
    pub async fn assemble_quote_with_options(
        &self,
        quote: SignedExternalQuote,
        options: AssembleQuoteOptions,
    ) -> Result<Option<ExternalMatchResponse>, ExternalMatchClientError> {
        let direction = quote.quote.order.side.clone();
        let v2_quote = v1_quote_to_v2(&quote);
        let v2_options = v1_assemble_options_to_v2(&options);
        let v2_resp = self.assemble_quote_with_options_v2(v2_quote, v2_options).await?;
        v2_resp.map(|r| v2_response_to_v1_non_malleable(r, direction)).transpose()
    }

    // ---------------------
    // | Match Routes (v1) |
    // ---------------------

    /// Request an external match (v1 API)
    ///
    /// Returns a non-malleable `ExternalMatchResponse`
    pub async fn request_external_match(
        &self,
        order: ExternalOrder,
    ) -> Result<Option<ExternalMatchResponse>, ExternalMatchClientError> {
        self.request_external_match_with_options(order, Default::default()).await
    }

    /// Request an external match with options (v1 API)
    ///
    /// Returns a non-malleable `ExternalMatchResponse`
    pub async fn request_external_match_with_options(
        &self,
        order: ExternalOrder,
        options: ExternalMatchOptions,
    ) -> Result<Option<ExternalMatchResponse>, ExternalMatchClientError> {
        let direction = order.side.clone();
        let v2_order = v1_order_to_v2(&order);
        let v2_resp = self.request_external_match_with_options_v2(v2_order, options).await?;
        v2_resp.map(|r| v2_response_to_v1_non_malleable(r, direction)).transpose()
    }
}
