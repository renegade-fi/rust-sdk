//! Request options types for the client
use url::form_urlencoded;

use crate::{
    api_types::{ExternalOrder, ASSEMBLE_MATCH_BUNDLE_ROUTE, GET_QUOTE_ROUTE},
    GAS_REFUND_NATIVE_ETH_QUERY_PARAM,
};

use super::{GAS_REFUND_ADDRESS_QUERY_PARAM, GAS_SPONSORSHIP_QUERY_PARAM};

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

        format!("{}?{}", GET_QUOTE_ROUTE, query.finish())
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

        format!("{}?{}", ASSEMBLE_MATCH_BUNDLE_ROUTE, query.finish())
    }
}

/// The options for assembling a quote
#[derive(Clone, Default)]
pub struct AssembleQuoteOptions {
    /// Whether to do gas estimation
    pub do_gas_estimation: bool,
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
}
