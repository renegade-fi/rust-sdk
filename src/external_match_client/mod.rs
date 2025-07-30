//! A client for requesting external matches from the relayer
//!
//! An external match is one between an internal party -- one with state
//! committed into the Renegade darkpool, and an external party -- one with no
//! state committed into the Renegade darkpool.

pub mod api_types;

mod client;
mod options;
use api_types::{Amount, ExternalOrder, OrderSide};
pub use client::ExternalMatchClient;
#[allow(deprecated)]
pub use options::{AssembleQuoteOptions, ExternalMatchOptions, RequestQuoteOptions};

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
    /// The exact base amount to output from the order
    exact_base_output: Option<Amount>,
    /// The exact quote amount to output from the order
    exact_quote_output: Option<Amount>,
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

    /// Set the exact base output amount
    pub fn exact_base_output(mut self, exact_base_output: Amount) -> Self {
        self.exact_base_output = Some(exact_base_output);
        self
    }

    /// Set the exact quote output amount
    pub fn exact_quote_output(mut self, exact_quote_output: Amount) -> Self {
        self.exact_quote_output = Some(exact_quote_output);
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

        // Ensure exactly one of the four amount fields is set
        let amount_fields_set = [
            self.base_amount.is_some(),
            self.quote_amount.is_some(),
            self.exact_base_output.is_some(),
            self.exact_quote_output.is_some(),
        ];
        let fields_set_count = amount_fields_set.iter().filter(|&&x| x).count();

        if fields_set_count == 0 {
            return Err(ExternalMatchClientError::invalid_order(
                "must set one of: `base_amount`, `quote_amount`, `exact_base_output`, or `exact_quote_output`",
            ));
        }
        if fields_set_count > 1 {
            return Err(ExternalMatchClientError::invalid_order(
                "can only set one of: `base_amount`, `quote_amount`, `exact_base_output`, or `exact_quote_output`",
            ));
        }

        let base_amount = self.base_amount.unwrap_or_default();
        let quote_amount = self.quote_amount.unwrap_or_default();
        let exact_base_output = self.exact_base_output.unwrap_or_default();
        let exact_quote_output = self.exact_quote_output.unwrap_or_default();

        let side = self.side.ok_or(ExternalMatchClientError::invalid_order("invalid side"))?;
        let min_fill_size = self.min_fill_size.unwrap_or_default();

        Ok(ExternalOrder {
            quote_mint,
            base_mint,
            base_amount,
            quote_amount,
            exact_base_output,
            exact_quote_output,
            side,
            min_fill_size,
        })
    }
}
