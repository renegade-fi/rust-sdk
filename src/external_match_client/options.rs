//! Request options types for the client
use url::form_urlencoded;

use crate::{
    api_types::{
        ASSEMBLE_EXTERNAL_MATCH_MALLEABLE_ROUTE, ASSEMBLE_EXTERNAL_MATCH_ROUTE,
        REQUEST_EXTERNAL_MATCH_ROUTE, REQUEST_EXTERNAL_QUOTE_ROUTE,
        REQUEST_MALLEABLE_EXTERNAL_MATCH_ROUTE,
    },
    types::ExternalOrder,
    GAS_REFUND_NATIVE_ETH_QUERY_PARAM,
};

use super::{
    GAS_REFUND_ADDRESS_QUERY_PARAM, GAS_SPONSORSHIP_QUERY_PARAM,
    USE_MALLEABLE_MATCH_CONNECTOR_QUERY_PARAM,
};

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
#[derive(Clone)]
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

impl Default for ExternalMatchOptions {
    fn default() -> Self {
        Self {
            do_gas_estimation: false,
            sponsor_gas: true,
            gas_refund_address: None,
            receiver_address: None,
        }
    }
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

    /// Get the request path for a malleable match given the options
    pub(crate) fn build_malleable_request_path(&self) -> String {
        let mut query = form_urlencoded::Serializer::new(String::new());

        // Add query params for gas sponsorship
        query.append_pair(GAS_SPONSORSHIP_QUERY_PARAM, &(!self.sponsor_gas).to_string());
        if let Some(addr) = &self.gas_refund_address {
            query.append_pair(GAS_REFUND_ADDRESS_QUERY_PARAM, addr);
        }

        format!("{}?{}", REQUEST_MALLEABLE_EXTERNAL_MATCH_ROUTE, query.finish())
    }
}

/// The options for assembling a quote
///
/// We attach the type parameter `USE_CONNECTOR` to the options type as it allow
/// us to statically:
/// - Limit the callsites at which `use_connector` may be applied; i.e. in
///   non-malleable matches we don't support the connector contract
/// - Statically define calldata parsing logic on the malleable match response
///   type for the connector ABI vs the gas sponsor ABI
#[derive(Clone, Default)]
pub struct AssembleQuoteOptions<const USE_CONNECTOR: bool> {
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

impl AssembleQuoteOptions<false> {
    /// Create a new options with default values
    pub fn new() -> Self {
        Default::default()
    }
}

impl<const USE_CONNECTOR: bool> AssembleQuoteOptions<USE_CONNECTOR> {
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

    /// Set whether to use the malleable match connector contract
    ///
    /// This contract allows the calldata to specify only the input amount,
    /// rather than the base and quote amounts. A different calldata type will
    /// be used and the response type methods will operate on this calldata
    /// in the appropriate way.
    #[allow(deprecated)]
    pub fn use_connector_contract(self) -> AssembleQuoteOptions<true> {
        AssembleQuoteOptions::<true> {
            do_gas_estimation: self.do_gas_estimation,
            allow_shared: self.allow_shared,
            sponsor_gas: self.sponsor_gas,
            gas_refund_address: self.gas_refund_address,
            receiver_address: self.receiver_address,
            updated_order: self.updated_order,
        }
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

    /// Get the request path for a malleable quote given the options
    #[allow(deprecated)]
    pub(crate) fn build_malleable_request_path(&self) -> String {
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

        if USE_CONNECTOR {
            query.append_pair(USE_MALLEABLE_MATCH_CONNECTOR_QUERY_PARAM, "true");
        }

        let query_str = query.finish();
        if query_str.is_empty() {
            return ASSEMBLE_EXTERNAL_MATCH_MALLEABLE_ROUTE.to_string();
        }

        format!("{ASSEMBLE_EXTERNAL_MATCH_MALLEABLE_ROUTE}?{query_str}")
    }
}
