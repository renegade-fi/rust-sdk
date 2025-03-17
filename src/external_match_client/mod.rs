//! A client for requesting external matches from the relayer
//!
//! An external match is one between an internal party -- one with state
//! committed into the Renegade darkpool, and an external party -- one with no
//! state committed into the Renegade darkpool.

pub mod api_types;

mod client;
use api_types::{Amount, ExternalOrder, OrderSide};
#[allow(deprecated)]
pub use client::{
    AssembleQuoteOptions, ExternalMatchClient, ExternalMatchOptions, RequestQuoteOptions,
};

mod error;
pub use error::ExternalMatchClientError;

/// The auth server query param for requesting gas sponsorship
pub const GAS_SPONSORSHIP_QUERY_PARAM: &str = "disable_gas_sponsorship";
/// The auth server query param for the gas refund address
pub const GAS_REFUND_ADDRESS_QUERY_PARAM: &str = "refund_address";
/// The auth server query param for refunding gas in terms of native ETH
pub const GAS_REFUND_NATIVE_ETH_QUERY_PARAM: &str = "refund_native_eth";

/// A builder for an [`ExternalOrder`]
#[derive(Debug, Clone, Default)]
pub struct ExternalOrderBuilder {
    /// The mint (erc20 address) of the quote token
    quote_mint: Option<String>,
    /// The mint (erc20 address) of the base token
    base_mint: Option<String>,
    /// The amount of the order
    base_amount: Option<Amount>,
    /// The amount of the order
    quote_amount: Option<Amount>,
    /// The side of the order
    side: Option<OrderSide>,
    /// The minimum fill size
    min_fill_size: Option<Amount>,
}

impl ExternalOrderBuilder {
    /// Create a new external order builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the quote mint
    ///
    /// Expects the quote mint as a hex encoded string
    pub fn quote_mint(mut self, quote_mint: &str) -> Self {
        self.quote_mint = Some(quote_mint.to_string());
        self
    }

    /// Set the base mint
    pub fn base_mint(mut self, base_mint: &str) -> Self {
        self.base_mint = Some(base_mint.to_string());
        self
    }

    /// Set the amount (deprecated -- use base_amount or quote_amount instead)
    #[deprecated(since = "0.1.0", note = "use base_amount() or quote_amount() instead")]
    pub fn amount(self, amount: Amount) -> Self {
        self.base_amount(amount)
    }

    /// Set the base amount
    pub fn base_amount(mut self, base_amount: Amount) -> Self {
        self.base_amount = Some(base_amount);
        self
    }

    /// Set the quote amount
    pub fn quote_amount(mut self, quote_amount: Amount) -> Self {
        self.quote_amount = Some(quote_amount);
        self
    }

    /// Set the side
    pub fn side(mut self, side: OrderSide) -> Self {
        self.side = Some(side);
        self
    }

    /// Set the minimum fill size
    pub fn min_fill_size(mut self, min_fill_size: Amount) -> Self {
        self.min_fill_size = Some(min_fill_size);
        self
    }

    /// Build the external order
    pub fn build(self) -> Result<ExternalOrder, ExternalMatchClientError> {
        let quote_mint =
            self.quote_mint.ok_or(ExternalMatchClientError::invalid_order("invalid quote mint"))?;
        let base_mint =
            self.base_mint.ok_or(ExternalMatchClientError::invalid_order("invalid base mint"))?;

        // Ensure exactly one of `base_amount` or `quote_amount` is set
        let (base_amount, quote_amount) = match (self.base_amount, self.quote_amount) {
            (Some(base), None) => (base, 0),
            (None, Some(quote)) => (0, quote),
            (None, None) => {
                return Err(ExternalMatchClientError::invalid_order(
                    "must set either `base_amount` or `quote_amount`",
                ))
            },
            (Some(_), Some(_)) => {
                return Err(ExternalMatchClientError::invalid_order(
                    "cannot set both `base_amount` and `quote_amount`",
                ))
            },
        };

        let side = self.side.ok_or(ExternalMatchClientError::invalid_order("invalid side"))?;
        let min_fill_size = self.min_fill_size.unwrap_or_default();

        Ok(ExternalOrder { quote_mint, base_mint, base_amount, quote_amount, side, min_fill_size })
    }
}
