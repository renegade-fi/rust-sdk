//! Type for operating on a malleable match result

use alloy::{primitives::U256, sol_types::SolValue};
use alloy_rpc_types_eth::{TransactionInput, TransactionRequest};

use crate::{types::NATIVE_ASSET_ADDR, ExternalMatchClientError};

use super::{MalleableExternalMatchResponse, OrderSide};

/// The offset of the base amount in the calldata
const BASE_AMOUNT_OFFSET: usize = 4;
/// The length of the base amount in the calldata
const BASE_AMOUNT_LENGTH: usize = 32;

/// The error emitted when a selected base amount is not in the valid range
const ERR_INVALID_BASE_AMOUNT: &str = "invalid base amount";

impl MalleableExternalMatchResponse {
    /// Get a settlement transaction with the current base amount
    pub fn settlement_tx(&self) -> TransactionRequest {
        self.match_bundle.settlement_tx.clone()
    }

    /// Set the `base_amount` of the `match_result`
    ///
    /// Returns the amount received at the given base amount
    pub fn set_base_amount(&mut self, base_amount: u128) -> Result<u128, ExternalMatchClientError> {
        self.check_base_amount(base_amount)?;

        // Set the calldata
        self.set_base_amount_calldata(base_amount);
        self.base_amount = Some(base_amount);
        Ok(self.receive_amount())
    }

    /// Set the calldata to use a given base amount
    pub fn set_base_amount_calldata(&mut self, base_amount: u128) {
        let mut modified_data = self.tx_data();
        let base_amt_slice =
            &mut modified_data[BASE_AMOUNT_OFFSET..BASE_AMOUNT_OFFSET + BASE_AMOUNT_LENGTH];

        let base_amount_u256 = U256::from(base_amount);
        let base_bytes = base_amount_u256.abi_encode();
        base_amt_slice.copy_from_slice(&base_bytes);

        let new_input = TransactionInput::new(modified_data.into());
        self.match_bundle.settlement_tx.input = new_input;

        // If the trade is a native ETH sell, we need to set the `value` of the tx
        if self.is_native_eth_sell() {
            self.match_bundle.settlement_tx.value = Some(U256::from(base_amount));
        }
    }

    /// Get the bounds on the base amount
    ///
    /// Returns a tuple [min_amount, max_amount] inclusive
    pub fn base_bounds(&self) -> (u128, u128) {
        (
            self.match_bundle.match_result.min_base_amount,
            self.match_bundle.match_result.max_base_amount,
        )
    }

    /// Get the receive amount at the currently set base amount
    pub fn receive_amount(&self) -> u128 {
        self.compute_receive_amount(self.current_base_amount())
    }

    /// Get the receive amount at the given base amount
    pub fn receive_amount_at_base(&self, base_amount: u128) -> u128 {
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
        let min = self.match_bundle.match_result.min_base_amount;
        let max = self.match_bundle.match_result.max_base_amount;
        if base_amount < min || base_amount > max {
            return Err(ExternalMatchClientError::invalid_modification(ERR_INVALID_BASE_AMOUNT));
        }

        Ok(())
    }

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

    /// Get the quote amount at the given base amount
    fn quote_amount(&self, base_amount: u128) -> u128 {
        self.match_bundle.match_result.price_fp.floor_mul_int(base_amount)
    }

    /// Get the current base amount
    fn current_base_amount(&self) -> u128 {
        self.base_amount.unwrap_or(self.match_bundle.match_result.max_base_amount)
    }

    /// Get the tx data
    fn tx_data(&self) -> Vec<u8> {
        let data = self.match_bundle.settlement_tx.input.input();
        data.unwrap_or_default().to_vec()
    }
}
