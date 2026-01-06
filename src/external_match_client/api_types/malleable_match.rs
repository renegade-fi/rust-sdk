//! Type for operating on a malleable match result

use alloy::{primitives::U256, sol_types::SolValue};
use alloy_rpc_types_eth::{TransactionInput, TransactionRequest};

use crate::{api_types::ExternalMatchResponse, ExternalMatchClientError};

/// The address used to represent the native asset
const NATIVE_ASSET_ADDR: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";

/// The offset of the input amount in the calldata, which is `4` because it's
/// the first calldata argument after the 4-byte function selector.
const INPUT_AMOUNT_OFFSET: usize = 4;
/// The length of an amount in calldata, which is 32 bytes for a `uint256`
const AMOUNT_CALLDATA_LENGTH: usize = 32;

/// The error emitted when a selected input amount is not in the valid range
const ERR_INVALID_INPUT_AMOUNT: &str = "invalid input amount";

impl ExternalMatchResponse {
    /// Get a settlement transaction with the current base amount
    pub fn settlement_tx(&self) -> TransactionRequest {
        self.match_bundle.settlement_tx.clone()
    }

    /// Get the tx data
    fn tx_data(&self) -> Vec<u8> {
        let data = self.match_bundle.settlement_tx.input.input();
        data.unwrap_or_default().to_vec()
    }

    // --- Bounds --- //

    /// Get the bounds on the input amount
    ///
    /// Returns a tuple [min_amount, max_amount] inclusive
    pub fn input_bounds(&self) -> (u128, u128) {
        (
            self.match_bundle.match_result.min_input_amount,
            self.match_bundle.match_result.max_input_amount,
        )
    }

    /// Get the bounds on the output amount
    ///
    /// Returns a tuple [min_amount, max_amount] inclusive
    pub fn output_bounds(&self) -> (u128, u128) {
        let (min_base, max_base) = self.input_bounds();
        let price = &self.match_bundle.match_result.output_quoted_price_fp;

        let min_output = price.floor_mul_int(min_base);
        let max_output = price.floor_mul_int(max_base);

        (min_output, max_output)
    }

    // --- Send and Receive Amounts --- //

    /// Get the current receive (output) amount at the given input amount
    ///
    /// This is net of fees
    fn compute_receive_amount(&self, input_amount: u128) -> u128 {
        let mut pre_sponsored_amt = self.output_amount(input_amount);

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

    /// Get the receive amount at the currently set input amount
    pub fn receive_amount(&self) -> u128 {
        self.compute_receive_amount(self.current_input_amount())
    }

    /// Get the receive amount at the given input amount
    pub fn receive_amount_at_base(&self, input_amount: u128) -> u128 {
        self.compute_receive_amount(input_amount)
    }

    /// Get the send amount at the currently set input amount
    pub fn send_amount(&self) -> u128 {
        self.current_input_amount()
    }

    // --- Input and Output Amounts --- //

    /// Get the output amount at the given input amount
    fn output_amount(&self, input_amount: u128) -> u128 {
        self.match_bundle.match_result.output_quoted_price_fp.floor_mul_int(input_amount)
    }

    /// Get the current input amount
    fn current_input_amount(&self) -> u128 {
        self.input_amount.unwrap_or(self.match_bundle.match_result.max_input_amount)
    }

    // --- Private Helpers --- //

    /// Whether the trade is a native ETH sell
    fn is_native_eth_sell(&self) -> bool {
        self.match_bundle.match_result.input_mint.to_lowercase() == NATIVE_ASSET_ADDR.to_lowercase()
    }

    /// Check an input amount is in the valid range
    fn check_input_amount(&self, input_amount: u128) -> Result<(), ExternalMatchClientError> {
        let (min, max) = self.input_bounds();
        if input_amount < min || input_amount > max {
            return Err(ExternalMatchClientError::invalid_modification(ERR_INVALID_INPUT_AMOUNT));
        }

        Ok(())
    }
}

impl ExternalMatchResponse {
    /// Set the input amount of the `match_result`. Returns the receive amount
    /// (output amount net of fees).
    pub fn set_input_amount(
        &mut self,
        input_amount: u128,
    ) -> Result<u128, ExternalMatchClientError> {
        self.check_input_amount(input_amount)?;

        // Set the calldata
        self.set_input_amount_calldata(input_amount);

        // Set the quote and base amounts on the response
        self.input_amount = Some(input_amount);
        Ok(self.receive_amount())
    }

    /// Set the calldata to use a given base amount
    pub fn set_input_amount_calldata(&mut self, input_amount: u128) {
        let mut modified_data = self.tx_data();
        let input_amt_slice =
            &mut modified_data[INPUT_AMOUNT_OFFSET..INPUT_AMOUNT_OFFSET + AMOUNT_CALLDATA_LENGTH];

        let input_amount_u256 = U256::from(input_amount);
        let input_bytes = input_amount_u256.abi_encode();
        input_amt_slice.copy_from_slice(&input_bytes);

        let new_input = TransactionInput::new(modified_data.into());
        self.match_bundle.settlement_tx.input = new_input;

        // If the trade is a native ETH sell, we need to set the `value` of the tx
        if self.is_native_eth_sell() {
            self.match_bundle.settlement_tx.value = Some(input_amount_u256);
        }
    }
}
