//! A client for requesting external matches from the relayer
//!
//! An external match is one between an internal party -- one with state
//! committed into the Renegade darkpool, and an external party -- one with no
//! state committed into the Renegade darkpool.

pub mod api_types;

mod client;
mod options;
mod v1_client;
mod v1_conversions;
use api_types::{Amount, ExternalOrderV2, OrderSide, v1_types};
pub use client::ExternalMatchClient;
#[allow(deprecated)]
pub use options::{
    AssembleQuoteOptions, AssembleQuoteOptionsV2, ExternalMatchOptions, RequestQuoteOptions,
};

mod error;
pub use error::ExternalMatchClientError;

/// The auth server query param for requesting gas sponsorship
pub const GAS_SPONSORSHIP_QUERY_PARAM: &str = "disable_gas_sponsorship";
/// The auth server query param for the gas refund address
pub const GAS_REFUND_ADDRESS_QUERY_PARAM: &str = "refund_address";
/// The auth server query param for refunding gas in terms of native ETH
pub const GAS_REFUND_NATIVE_ETH_QUERY_PARAM: &str = "refund_native_eth";
/// A builder for an [`ExternalOrderV2`]
#[derive(Debug, Clone, Default)]
pub struct ExternalOrderBuilderV2 {
    /// The mint (erc20 address) of the input token
    input_mint: Option<String>,
    /// The mint (erc20 address) of the output token
    output_mint: Option<String>,
    /// The input amount of the order
    input_amount: Option<Amount>,
    /// The output amount of the order
    output_amount: Option<Amount>,
    /// Whether to consider the output amount as an exact amount (net of fees)
    use_exact_output_amount: Option<bool>,
    /// The minimum fill size
    min_fill_size: Option<Amount>,
}

impl ExternalOrderBuilderV2 {
    /// Create a new external order builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the input mint
    ///
    /// Expects the input mint as a hex encoded string
    pub fn input_mint(mut self, input_mint: &str) -> Self {
        self.input_mint = Some(input_mint.to_string());
        self
    }

    /// Set the output mint
    pub fn output_mint(mut self, output_mint: &str) -> Self {
        self.output_mint = Some(output_mint.to_string());
        self
    }

    /// Set the input amount
    pub fn input_amount(mut self, input_amount: Amount) -> Self {
        self.input_amount = Some(input_amount);
        self
    }

    /// Set the output amount
    pub fn output_amount(mut self, output_amount: Amount) -> Self {
        self.output_amount = Some(output_amount);
        self
    }

    /// Set whether to use an exact output amount
    pub fn use_exact_output_amount(mut self) -> Self {
        self.use_exact_output_amount = Some(true);
        self
    }

    /// Set the minimum fill size
    pub fn min_fill_size(mut self, min_fill_size: Amount) -> Self {
        self.min_fill_size = Some(min_fill_size);
        self
    }

    /// Build the external order
    pub fn build(self) -> Result<ExternalOrderV2, ExternalMatchClientError> {
        let input_mint =
            self.input_mint.ok_or(ExternalMatchClientError::invalid_order("invalid input mint"))?;

        let output_mint = self
            .output_mint
            .ok_or(ExternalMatchClientError::invalid_order("invalid output mint"))?;

        // Ensure exactly one of the amount fields is set
        let input_zero = self.input_amount.is_none_or(|amt| amt == 0);
        let output_zero = self.output_amount.is_none_or(|amt| amt == 0);
        if !(input_zero ^ output_zero) {
            return Err(ExternalMatchClientError::invalid_order(
                "exactly one of input_amount or output_amount must be set",
            ));
        }

        let input_amount = self.input_amount.unwrap_or_default();
        let output_amount = self.output_amount.unwrap_or_default();
        let use_exact_output_amount = self.use_exact_output_amount.unwrap_or_default();
        let min_fill_size = self.min_fill_size.unwrap_or_default();

        Ok(ExternalOrderV2 {
            input_mint,
            output_mint,
            input_amount,
            output_amount,
            use_exact_output_amount,
            min_fill_size,
        })
    }
}

/// A builder for an [`ExternalOrder`](v1_types::ExternalOrder) (v1 format)
#[derive(Debug, Clone, Default)]
pub struct ExternalOrderBuilder {
    /// The mint (erc20 address) of the quote token
    quote_mint: Option<String>,
    /// The mint (erc20 address) of the base token
    base_mint: Option<String>,
    /// The base amount of the order
    base_amount: Option<Amount>,
    /// The quote amount of the order
    quote_amount: Option<Amount>,
    /// The exact base output amount
    exact_base_output: Option<Amount>,
    /// The exact quote output amount
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
    pub fn build(self) -> Result<v1_types::ExternalOrder, ExternalMatchClientError> {
        let quote_mint =
            self.quote_mint.ok_or(ExternalMatchClientError::invalid_order("invalid quote mint"))?;
        let base_mint =
            self.base_mint.ok_or(ExternalMatchClientError::invalid_order("invalid base mint"))?;

        // Ensure exactly one of the four amount fields is set
        let base_zero = self.base_amount.is_none_or(|amt| amt == 0);
        let quote_zero = self.quote_amount.is_none_or(|amt| amt == 0);
        let exact_base_output = self.exact_base_output.is_some_and(|amt| amt != 0);
        let exact_quote_output = self.exact_quote_output.is_some_and(|amt| amt != 0);

        let n_sizes_set = (!base_zero as u8)
            + (!quote_zero as u8)
            + (exact_base_output as u8)
            + (exact_quote_output as u8);

        if n_sizes_set != 1 {
            return Err(ExternalMatchClientError::invalid_order(
                "exactly one of base_amount, quote_amount, exact_base_output, or exact_quote_output must be set",
            ));
        }

        let base_amount = self.base_amount.unwrap_or_default();
        let quote_amount = self.quote_amount.unwrap_or_default();
        let exact_base_output = self.exact_base_output.unwrap_or_default();
        let exact_quote_output = self.exact_quote_output.unwrap_or_default();

        let side = self.side.ok_or(ExternalMatchClientError::invalid_order("invalid side"))?;
        let min_fill_size = self.min_fill_size.unwrap_or_default();

        Ok(v1_types::ExternalOrder {
            quote_mint,
            base_mint,
            side,
            base_amount,
            quote_amount,
            exact_base_output,
            exact_quote_output,
            min_fill_size,
        })
    }
}
