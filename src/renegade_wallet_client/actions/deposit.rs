//! Deposit into an account balance

use alloy::primitives::Address;
use renegade_circuit_types::Amount;
use renegade_crypto::fields::scalar_to_u256;
use renegade_darkpool_types::balance::DarkpoolBalance;
use renegade_darkpool_types::balance::DarkpoolStateBalance;
use renegade_solidity_abi::v2::{
    IDarkpoolV2, relayer_types::u128_to_u256, transfer_auth::deposit::create_deposit_permit,
};

use crate::{
    RenegadeClientError,
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        DEPOSIT_BALANCE_ROUTE,
        balances::{ApiBalance, ApiDepositPermit},
        request_response::{
            DepositBalanceQueryParameters, DepositBalanceRequest, DepositBalanceResponse,
        },
    },
    websocket::{DEFAULT_TASK_TIMEOUT, TaskWaiter},
};

// --- Public Actions --- //
impl RenegadeClient {
    /// Deposit funds into an account balance. Waits for the deposit task to
    /// complete before returning the post-deposit balance.
    pub async fn deposit(
        &self,
        mint: Address,
        amount: Amount,
    ) -> Result<ApiBalance, RenegadeClientError> {
        let request = self.build_deposit_request(mint, amount).await?;

        let query_params = DepositBalanceQueryParameters { non_blocking: Some(false) };
        let path = self.build_deposit_request_path(mint, &query_params)?;

        let DepositBalanceResponse { balance, .. } =
            self.relayer_client.post(&path, request).await?;

        Ok(balance)
    }

    /// Enqueues a deposit task in the relayer. Returns the post-deposit
    /// balance, and a `TaskWaiter` that can be used to await task
    /// completion.
    pub async fn enqueue_deposit(
        &self,
        mint: Address,
        amount: Amount,
    ) -> Result<(ApiBalance, TaskWaiter), RenegadeClientError> {
        let request = self.build_deposit_request(mint, amount).await?;

        let query_params = DepositBalanceQueryParameters { non_blocking: Some(false) };
        let path = self.build_deposit_request_path(mint, &query_params)?;

        let DepositBalanceResponse { balance, task_id, .. } =
            self.relayer_client.post(&path, request).await?;

        let task_waiter = self.watch_task(task_id, DEFAULT_TASK_TIMEOUT).await?;

        Ok((balance, task_waiter))
    }
}

// --- Private Helpers --- //
impl RenegadeClient {
    /// Builds the request to deposit a balance
    async fn build_deposit_request(
        &self,
        mint: Address,
        amount: Amount,
    ) -> Result<DepositBalanceRequest, RenegadeClientError> {
        let permit = self.build_deposit_permit(mint, amount).await?;

        let from_address = self.get_account_address();
        let authority = self.get_schnorr_public_key().into();

        Ok(DepositBalanceRequest { from_address, amount, authority, permit })
    }

    /// Builds the request to deposit a balance
    async fn build_deposit_permit(
        &self,
        mint: Address,
        amount: Amount,
    ) -> Result<ApiDepositPermit, RenegadeClientError> {
        // First, we check if a balance already exists for the token being deposited
        let existing_balance = self.get_balance_by_mint(mint).await;

        let state_balance = if let Ok(balance) = existing_balance {
            // If a balance already exists for the token being deposited,
            // we update its amount, progressing its cryptographic state accordingly.

            let mut state_balance: DarkpoolStateBalance = balance.into();

            state_balance.inner.amount += amount;
            let new_amount = state_balance.inner.amount;
            let new_amount_public_share = state_balance.stream_cipher_encrypt(&new_amount);
            state_balance.public_share.amount = new_amount_public_share;

            state_balance.compute_recovery_id();

            state_balance
        } else {
            // If this is a deposit into a new balance, we create the balance state object &
            // progress its cryptographic state accordingly.

            let balance = DarkpoolBalance::new(
                mint,
                self.get_account_address(),
                self.get_relayer_fee_recipient(),
                self.get_schnorr_public_key(),
            )
            .with_amount(amount);

            let (mut recovery_seed_csprng, mut share_seed_csprng) =
                self.get_account_seeds().await?;

            let balance_recovery_stream_seed = recovery_seed_csprng.next().unwrap();
            let balance_share_stream_seed = share_seed_csprng.next().unwrap();

            let mut state_balance = DarkpoolStateBalance::new(
                balance,
                balance_share_stream_seed,
                balance_recovery_stream_seed,
            );

            state_balance.compute_recovery_id();

            state_balance
        };

        let commitment = scalar_to_u256(&state_balance.compute_commitment());

        let deposit = IDarkpoolV2::Deposit {
            from: self.get_account_address(),
            token: mint,
            amount: u128_to_u256(amount),
        };

        let (witness, signature) = create_deposit_permit(
            commitment,
            deposit,
            self.get_chain_id(),
            self.get_darkpool_address(),
            self.get_permit2_address(),
            self.get_account_signer(),
        )
        .map_err(RenegadeClientError::signing)?;

        Ok(ApiDepositPermit {
            nonce: witness.nonce,
            deadline: witness.deadline,
            signature: signature.into(),
        })
    }

    /// Builds the request path for the deposit balance endpoint
    fn build_deposit_request_path(
        &self,
        mint: Address,
        query_params: &DepositBalanceQueryParameters,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(DEPOSIT_BALANCE_ROUTE, "account_id" => self.get_account_id(), "mint" => mint);
        let query_string =
            serde_urlencoded::to_string(query_params).map_err(RenegadeClientError::serde)?;

        Ok(format!("{path}?{query_string}"))
    }
}
