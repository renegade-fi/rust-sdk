//! Withdraw funds from an account balance

use std::time::Duration;

use alloy::primitives::Address;
use renegade_circuit_types::Amount;
use renegade_crypto::fields::scalar_to_u256;
use renegade_darkpool_types::balance::DarkpoolStateBalance;
use renegade_solidity_abi::v2::{
    IDarkpoolV2::WithdrawalAuth, transfer_auth::withdrawal::create_withdrawal_auth,
};

use crate::{
    RenegadeClientError,
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        WITHDRAW_BALANCE_ROUTE,
        balances::ApiBalance,
        request_response::{
            WithdrawBalanceQueryParameters, WithdrawBalanceRequest, WithdrawBalanceResponse,
        },
    },
    websocket::TaskWaiter,
};

/// The timeout for a withdrawal action to complete.
///
/// This is longer than the default since any enqueued fee payment tasks must
/// complete first.
const TASK_WAITER_TIMEOUT: Duration = Duration::from_secs(120);

// --- Public Actions --- //
impl RenegadeClient {
    /// Withdraw funds from an account balance. Waits for the withdrawal task to
    /// complete before returning the post-withdrawal balance.
    pub async fn withdraw(
        &self,
        mint: Address,
        amount: Amount,
    ) -> Result<ApiBalance, RenegadeClientError> {
        let request = self.build_withdrawal_request(mint, amount).await?;

        let query_params = WithdrawBalanceQueryParameters { non_blocking: Some(false) };
        let path = self.build_withdrawal_request_path(mint, &query_params)?;

        let WithdrawBalanceResponse { balance, .. } =
            self.relayer_client.post(&path, request).await?;

        Ok(balance)
    }

    /// Enqueues a withdrawal task in the relayer. Returns the post-withdrawal
    /// balance, and a `TaskWaiter` that can be used to await task completion.
    pub async fn enqueue_withdrawal(
        &self,
        mint: Address,
        amount: Amount,
    ) -> Result<(ApiBalance, TaskWaiter), RenegadeClientError> {
        let request = self.build_withdrawal_request(mint, amount).await?;

        let query_params = WithdrawBalanceQueryParameters { non_blocking: Some(true) };
        let path = self.build_withdrawal_request_path(mint, &query_params)?;

        let WithdrawBalanceResponse { balance, task_id, .. } =
            self.relayer_client.post(&path, request).await?;

        let task_waiter = self.watch_task(task_id, TASK_WAITER_TIMEOUT).await?;

        Ok((balance, task_waiter))
    }
}

// --- Private Helpers --- //
impl RenegadeClient {
    /// Builds the request to withdraw from a balance
    async fn build_withdrawal_request(
        &self,
        mint: Address,
        amount: Amount,
    ) -> Result<WithdrawBalanceRequest, RenegadeClientError> {
        let signature = self.build_withdrawal_auth(mint, amount).await?;
        Ok(WithdrawBalanceRequest { amount, signature })
    }

    /// Builds the signature over the balance commitment which authorizes the
    /// withdrawal
    async fn build_withdrawal_auth(
        &self,
        mint: Address,
        amount: Amount,
    ) -> Result<Vec<u8>, RenegadeClientError> {
        let balance = self.get_balance_by_mint(mint).await?;
        let mut state_balance: DarkpoolStateBalance = balance.into();

        // First, we simulate fee payments on the balance.
        // This is necessary because the withdrawal API handler will execute fee
        // payments before executing the withdrawal.
        // We need to ensure that the balance whose commitment we sign to authorize the
        // withdrawal is correctly updated to reflect this.
        simulate_fee_payments(&mut state_balance);

        // Next, we update the balance's amount, progressing its cryptographic state
        // accordingly.
        state_balance.inner.amount -= amount;
        let new_amount = state_balance.inner.amount;
        let new_amount_public_share = state_balance.stream_cipher_encrypt(&new_amount);
        state_balance.public_share.amount = new_amount_public_share;
        state_balance.compute_recovery_id();

        // Finally, we compute the commitment to the balance & sign it to authorize the
        // withdrawal
        let commitment = scalar_to_u256(&state_balance.compute_commitment());
        let chain_id = self.get_chain_id();

        let WithdrawalAuth { signature } =
            create_withdrawal_auth(commitment, chain_id, self.get_account_signer())
                .map_err(RenegadeClientError::signing)?;

        Ok(signature.to_vec())
    }

    /// Builds the request path for the withdrawal balance endpoint
    fn build_withdrawal_request_path(
        &self,
        mint: Address,
        query_params: &WithdrawBalanceQueryParameters,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(WITHDRAW_BALANCE_ROUTE, "account_id" => self.get_account_id(), "mint" => mint);
        let query_string =
            serde_urlencoded::to_string(query_params).map_err(RenegadeClientError::serde)?;

        Ok(format!("{path}?{query_string}"))
    }
}

// ----------------------
// | Non-Member Helpers |
// ----------------------

/// Simulates fee payments on the balance
fn simulate_fee_payments(state_balance: &mut DarkpoolStateBalance) {
    // First, we simulate the relayer fee payment
    state_balance.pay_relayer_fee();
    state_balance.reencrypt_relayer_fee();
    state_balance.compute_recovery_id();

    // Then, we simulate the protocol fee payment.
    // We use a dummy address for the protocol fee receiver since we don't actually
    // need a valid fee note.
    state_balance.pay_protocol_fee(Address::ZERO);
    state_balance.reencrypt_protocol_fee();
    state_balance.compute_recovery_id();
}
