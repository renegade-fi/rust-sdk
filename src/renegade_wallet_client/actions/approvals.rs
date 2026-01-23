//! Helper methods for building ERC20 and Permit2 approval transactions

use alloy::{
    primitives::{Address, U160, U256, aliases::U48},
    sol_types::SolCall,
};
use alloy_rpc_types_eth::{TransactionInput, TransactionRequest};

use crate::{
    client::RenegadeClient,
    renegade_wallet_client::utils::{IAllowanceTransfer, approveCall},
};

impl RenegadeClient {
    /// Build a transaction to approve the Permit2 contract as a spender for
    /// the given ERC20 token.
    ///
    /// # Arguments
    /// * `token` - The ERC20 token address to approve
    /// * `amount` - The amount to approve for spending
    ///
    /// # Returns
    /// A `TransactionRequest` that can be executed by the user with their
    /// provider
    pub fn build_erc20_approval_tx(&self, token: Address, amount: U256) -> TransactionRequest {
        let calldata = approveCall { spender: self.get_permit2_address(), amount }.abi_encode();
        TransactionRequest::default().to(token).input(TransactionInput::new(calldata.into()))
    }

    /// Build a transaction to approve the darkpool as a spender through
    /// Permit2's AllowanceTransfer interface.
    ///
    /// # Arguments
    /// * `token` - The ERC20 token address to approve
    /// * `amount` - The amount to approve for spending (uint160)
    /// * `expiration` - The Unix timestamp when this approval expires (uint48)
    ///
    /// # Returns
    /// A `TransactionRequest` that can be executed by the user with their
    /// provider
    pub fn build_permit2_allowance_tx(
        &self,
        token: Address,
        amount: U160,
        expiration: U48,
    ) -> TransactionRequest {
        let calldata = IAllowanceTransfer::approveCall {
            token,
            spender: self.get_darkpool_address(),
            amount,
            expiration,
        }
        .abi_encode();
        TransactionRequest::default()
            .to(self.get_permit2_address())
            .input(TransactionInput::new(calldata.into()))
    }
}
