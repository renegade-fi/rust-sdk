//! Type for operating on a malleable match result

use alloy::{primitives::U256, sol_types::SolValue};
use alloy_rpc_types_eth::{TransactionInput, TransactionRequest};

use crate::{
    api_types::GenericMalleableExternalMatchResponse, types::NATIVE_ASSET_ADDR,
    ExternalMatchClientError,
};

use super::{FixedPoint, OrderSide};

/// The offset of the quote amount in the calldata,
/// which is `4` because it's the first calldata argument after the 4-byte
/// function selector.
const QUOTE_AMOUNT_OFFSET: usize = 4;
/// The offset of the base amount in the calldata, which is
/// `AMOUNT_CALLDATA_LENGTH` after the quote amount as it is the next calldata
/// argument.
const BASE_AMOUNT_OFFSET: usize = QUOTE_AMOUNT_OFFSET + AMOUNT_CALLDATA_LENGTH;
/// The offset of the input amount in the calldata, which is `4` because it's
/// the first calldata argument after the 4-byte function selector.
const INPUT_AMOUNT_OFFSET: usize = 4;
/// The length of an amount in calldata, which is 32 bytes for a `uint256`
const AMOUNT_CALLDATA_LENGTH: usize = 32;

/// The error emitted when a selected base amount is not in the valid range
const ERR_INVALID_BASE_AMOUNT: &str = "invalid base amount";
/// The error emitted when a selected quote amount is not in the valid range
const ERR_INVALID_QUOTE_AMOUNT: &str = "invalid quote amount";

// Connector-agnostic methods on the malleable match response
impl<const USE_CONNECTOR: bool> GenericMalleableExternalMatchResponse<USE_CONNECTOR> {
    /// Get a settlement transaction with the current base amount
    pub fn settlement_tx(&self) -> TransactionRequest {
        self.match_bundle.settlement_tx.clone()
    }

    /// Get the tx data
    fn tx_data(&self) -> Vec<u8> {
        let data = self.match_bundle.settlement_tx.input.input();
        data.unwrap_or_default().to_vec()
    }

    /// Whether the trade sells the base token
    pub fn sells_base_token(&self) -> bool {
        self.match_bundle.match_result.direction == OrderSide::Sell
    }

    // --- Bounds --- //

    /// Get the bounds on the base amount
    ///
    /// Returns a tuple [min_amount, max_amount] inclusive
    pub fn base_bounds(&self) -> (u128, u128) {
        (
            self.match_bundle.match_result.min_base_amount,
            self.match_bundle.match_result.max_base_amount,
        )
    }

    /// Get the bounds on the quote amount
    ///
    /// Returns a tuple [min_amount, max_amount] inclusive
    pub fn quote_bounds(&self) -> (u128, u128) {
        let (min_base, max_base) = self.base_bounds();
        let price = &self.match_bundle.match_result.price_fp;

        let min_quote = price.floor_mul_int(min_base);
        let max_quote = price.floor_mul_int(max_base);

        (min_quote, max_quote)
    }

    /// Get the bounds on the quote amount for a given base amount.
    ///
    /// For an explanation of these bounds, see:
    /// https://github.com/renegade-fi/renegade-contracts/blob/main/contracts-common/src/types/match.rs#L144-L174
    pub fn quote_bounds_for_base(&self, base_amount: u128) -> (u128, u128) {
        let (min_quote, max_quote) = self.quote_bounds();

        let price = &self.match_bundle.match_result.price_fp;
        let ref_quote = price.floor_mul_int(base_amount);

        let (range_min, range_max) = match self.match_bundle.match_result.direction {
            OrderSide::Buy => (ref_quote, max_quote),
            OrderSide::Sell => (min_quote, ref_quote),
        };

        (range_min, range_max)
    }

    // --- Send and Receive Amounts --- //

    /// Get the current receive amount at the given base amount
    ///
    /// This is net of fees
    fn compute_receive_amount(&self, base_amount: u128) -> u128 {
        let match_res = &self.match_bundle.match_result;
        let mut pre_sponsored_amt = match match_res.direction {
            OrderSide::Buy => base_amount,
            OrderSide::Sell => self.quote_amount(base_amount),
        };

        // Account for fees
        let total_fee = self.match_bundle.fee_rates.total();
        let total_fee_amount = total_fee.floor_mul_int(pre_sponsored_amt);
        pre_sponsored_amt -= total_fee_amount;

        // Account for gas sponsorship
        if let Some(info) = &self.gas_sponsorship_info {
            if !info.refund_native_eth {
                pre_sponsored_amt += info.refund_amount;
            }
        };

        pre_sponsored_amt
    }

    /// Get the current send amount at the given base amount
    fn compute_send_amount(&self, base_amount: u128) -> u128 {
        let match_res = &self.match_bundle.match_result;
        match match_res.direction {
            OrderSide::Buy => self.quote_amount(base_amount),
            OrderSide::Sell => base_amount,
        }
    }

    /// Get the receive amount at the currently set base amount
    pub fn receive_amount(&self) -> u128 {
        self.compute_receive_amount(self.current_base_amount())
    }

    /// Get the receive amount at the given base amount
    pub fn receive_amount_at_base(&self, base_amount: u128) -> u128 {
        self.compute_receive_amount(base_amount)
    }

    /// Get the receive amount at the given quote amount
    pub fn receive_amount_at_quote(&self, quote_amount: u128) -> u128 {
        let base_amount = self.base_amount(quote_amount);
        self.compute_receive_amount(base_amount)
    }

    /// Get the send amount at the currently set base amount
    pub fn send_amount(&self) -> u128 {
        self.compute_send_amount(self.current_base_amount())
    }

    /// Get the send amount at the given base amount
    pub fn send_amount_at_base(&self, base_amount: u128) -> u128 {
        self.compute_send_amount(base_amount)
    }

    /// Get the send amount at the given quote amount
    pub fn send_amount_at_quote(&self, quote_amount: u128) -> u128 {
        let base_amount = self.base_amount(quote_amount);
        self.compute_send_amount(base_amount)
    }

    // --- Base and Quote Amounts --- //

    /// Get the base amount at the given quote amount
    fn base_amount(&self, quote_amount: u128) -> u128 {
        FixedPoint::ceil_div_int(quote_amount, &self.match_bundle.match_result.price_fp)
    }

    /// Get the quote amount at the given base amount
    fn quote_amount(&self, base_amount: u128) -> u128 {
        self.match_bundle.match_result.price_fp.floor_mul_int(base_amount)
    }

    /// Get the current base amount
    fn current_base_amount(&self) -> u128 {
        self.base_amount.unwrap_or(self.match_bundle.match_result.max_base_amount)
    }

    // --- Private Helpers --- //

    /// Whether the trade is a native ETH sell
    fn is_native_eth_sell(&self) -> bool {
        let match_res = &self.match_bundle.match_result;
        let is_sell = match_res.direction == OrderSide::Sell;
        let is_base_eth = match_res.base_mint.to_lowercase() == NATIVE_ASSET_ADDR.to_lowercase();

        is_base_eth && is_sell
    }

    /// Check a base amount is in the valid range
    fn check_base_amount(&self, base_amount: u128) -> Result<(), ExternalMatchClientError> {
        let (min, max) = self.base_bounds();
        if base_amount < min || base_amount > max {
            return Err(ExternalMatchClientError::invalid_modification(ERR_INVALID_BASE_AMOUNT));
        }

        Ok(())
    }

    /// Check a quote amount is in the valid range for a given base amount.
    ///
    /// This is true if the quote amount is within the bounds implied by the min
    /// and max base amounts given the price in the match results, and the
    /// quote amount does not imply a price improvement over the price in
    /// the match result.
    fn check_quote_amount(
        &self,
        quote_amount: u128,
        base_amount: u128,
    ) -> Result<(), ExternalMatchClientError> {
        let (min_quote, max_quote) = self.quote_bounds_for_base(base_amount);
        if quote_amount < min_quote || quote_amount > max_quote {
            return Err(ExternalMatchClientError::invalid_modification(ERR_INVALID_QUOTE_AMOUNT));
        }

        Ok(())
    }
}

// -----------------------------------
// | Gas Sponsor ABI Implementations |
// -----------------------------------

// Implementations for a malleable match routed through the gas sponsor ABI
impl GenericMalleableExternalMatchResponse<false /* USE_CONNECTOR */> {
    /// Set the `base_amount` of the `match_result`
    ///
    /// Returns the amount received at the given base amount
    pub fn set_base_amount(&mut self, base_amount: u128) -> Result<u128, ExternalMatchClientError> {
        self.check_base_amount(base_amount)?;

        let implied_quote_amount = self.quote_amount(base_amount);

        // Set the calldata
        self.set_base_amount_calldata(base_amount);
        self.set_quote_amount_calldata(implied_quote_amount);

        // Set the quote and base amounts on the response
        self.base_amount = Some(base_amount);
        self.quote_amount = Some(implied_quote_amount);

        Ok(self.receive_amount())
    }

    /// Set the calldata to use a given base amount
    pub fn set_base_amount_calldata(&mut self, base_amount: u128) {
        let mut modified_data = self.tx_data();
        let base_amt_slice =
            &mut modified_data[BASE_AMOUNT_OFFSET..BASE_AMOUNT_OFFSET + AMOUNT_CALLDATA_LENGTH];

        let base_amount_u256 = U256::from(base_amount);
        let base_bytes = base_amount_u256.abi_encode();
        base_amt_slice.copy_from_slice(&base_bytes);

        let new_input = TransactionInput::new(modified_data.into());
        self.match_bundle.settlement_tx.input = new_input;

        // If the trade is a native ETH sell, we need to set the `value` of the tx
        if self.is_native_eth_sell() {
            self.match_bundle.settlement_tx.value = Some(base_amount_u256);
        }
    }

    /// Set the `quote_amount` of the `match_result`
    ///
    /// Returns the amount received at the given quote amount
    pub fn set_quote_amount(
        &mut self,
        quote_amount: u128,
    ) -> Result<u128, ExternalMatchClientError> {
        let implied_base_amount = self.base_amount(quote_amount);
        self.check_quote_amount(quote_amount, implied_base_amount)?;

        // Set the calldata
        self.set_quote_amount_calldata(quote_amount);
        self.set_base_amount_calldata(implied_base_amount);

        // Set the quote and base amounts on the response
        self.quote_amount = Some(quote_amount);
        self.base_amount = Some(implied_base_amount);

        Ok(self.receive_amount())
    }

    /// Set the calldata to use a given quote amount
    pub fn set_quote_amount_calldata(&mut self, quote_amount: u128) {
        let mut modified_data = self.tx_data();
        let quote_amt_slice =
            &mut modified_data[QUOTE_AMOUNT_OFFSET..QUOTE_AMOUNT_OFFSET + AMOUNT_CALLDATA_LENGTH];

        let quote_amount_u256 = U256::from(quote_amount);
        let quote_bytes = quote_amount_u256.abi_encode();
        quote_amt_slice.copy_from_slice(&quote_bytes);

        let new_input = TransactionInput::new(modified_data.into());
        self.match_bundle.settlement_tx.input = new_input;
    }
}

// ---------------------------------
// | Connector ABI Implementations |
// ---------------------------------

// Implementations for a malleable match routed through the connector ABI
impl GenericMalleableExternalMatchResponse<true /* USE_CONNECTOR */> {
    /// Set the input amount of the `match_result`
    pub fn set_input_amount(
        &mut self,
        input_amount: u128,
    ) -> Result<u128, ExternalMatchClientError> {
        if self.sells_base_token() {
            self.set_base_amount(input_amount)
        } else {
            self.set_quote_amount(input_amount)
        }
    }

    /// Set the `base_amount` of the `match_result`
    ///
    /// Returns the amount received at the given base amount
    pub fn set_base_amount(&mut self, base_amount: u128) -> Result<u128, ExternalMatchClientError> {
        self.check_base_amount(base_amount)?;
        let implied_quote_amount = self.quote_amount(base_amount);

        // Set the calldata
        self.set_base_amount_calldata(base_amount);
        self.set_quote_amount_calldata(implied_quote_amount);

        // Set the quote and base amounts on the response
        self.base_amount = Some(base_amount);
        self.quote_amount = Some(implied_quote_amount);
        Ok(self.receive_amount())
    }

    /// Set the calldata to use a given base amount
    pub fn set_base_amount_calldata(&mut self, base_amount: u128) {
        // If the trade does not sell the base token, we don't need to modify calldata
        // Only the _input_ amount is set in the calldata for the connector ABI
        if !self.sells_base_token() {
            return;
        }

        let mut modified_data = self.tx_data();
        let input_amt_slice =
            &mut modified_data[INPUT_AMOUNT_OFFSET..INPUT_AMOUNT_OFFSET + AMOUNT_CALLDATA_LENGTH];

        let base_amount_u256 = U256::from(base_amount);
        let base_bytes = base_amount_u256.abi_encode();
        input_amt_slice.copy_from_slice(&base_bytes);

        let new_input = TransactionInput::new(modified_data.into());
        self.match_bundle.settlement_tx.input = new_input;

        // If the trade is a native ETH sell, we need to set the `value` of the tx
        if self.is_native_eth_sell() {
            self.match_bundle.settlement_tx.value = Some(base_amount_u256);
        }
    }

    /// Set the `quote_amount` of the `match_result`
    ///
    /// Returns the amount received at the given quote amount
    pub fn set_quote_amount(
        &mut self,
        quote_amount: u128,
    ) -> Result<u128, ExternalMatchClientError> {
        let implied_base_amount = self.base_amount(quote_amount);
        self.check_quote_amount(quote_amount, implied_base_amount)?;

        // Set the calldata
        self.set_quote_amount_calldata(quote_amount);
        self.set_base_amount_calldata(implied_base_amount);

        // Set the quote and base amounts on the response
        self.quote_amount = Some(quote_amount);
        self.base_amount = Some(implied_base_amount);

        Ok(self.receive_amount())
    }

    /// Set the calldata to use a given quote amount
    pub fn set_quote_amount_calldata(&mut self, quote_amount: u128) {
        // If the trade sells the base token, we do not need to modify calldata
        // Only the _quote_ amount is set in the calldata for the connector ABI
        if self.sells_base_token() {
            return;
        }

        let mut modified_data = self.tx_data();
        let input_amt_slice =
            &mut modified_data[INPUT_AMOUNT_OFFSET..INPUT_AMOUNT_OFFSET + AMOUNT_CALLDATA_LENGTH];

        let quote_amount_u256 = U256::from(quote_amount);
        let quote_bytes = quote_amount_u256.abi_encode();
        input_amt_slice.copy_from_slice(&quote_bytes);

        let new_input = TransactionInput::new(modified_data.into());
        self.match_bundle.settlement_tx.input = new_input;
    }
}
