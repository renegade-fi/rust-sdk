//! Deposit into an account balance

use alloy::primitives::Address;
use renegade_circuit_types::{
    balance::{Balance, DarkpoolStateBalance},
    Amount,
};
use renegade_crypto::fields::scalar_to_u256;
use renegade_solidity_abi::v2::{
    relayer_types::u128_to_u256, transfer_auth::deposit::create_deposit_permit, IDarkpoolV2,
};

use crate::{
    actions::construct_http_path,
    client::RenegadeClient,
    renegade_api_types::{
        balances::{ApiBalance, ApiDepositPermit},
        request_response::{
            DepositBalanceQueryParameters, DepositBalanceRequest, DepositBalanceResponse,
        },
        DEPOSIT_BALANCE_ROUTE,
    },
    websocket::TaskWaiter,
    RenegadeClientError,
};

// --- Public Actions --- //
impl RenegadeClient {
    /// Deposit funds into an account balance. Waits for the deposit task to
    /// complete before returning the deposited balance.
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

    /// Enqueues a deposit task in the relayer. Returns the deposited balance,
    /// and a `TaskWaiter` that can be used to await task completion.
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

        Ok((balance, self.get_default_task_waiter(task_id)))
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
            balance.into()
        } else {
            let balance = Balance::new(
                mint,
                self.get_account_address(),
                self.get_relayer_fee_recipient(),
                self.get_schnorr_public_key(),
            );

            let (mut recovery_seed_csprng, mut share_seed_csprng) =
                self.get_account_seeds().await?;

            let balance_recovery_stream_seed = recovery_seed_csprng.next().unwrap();
            let balance_share_stream_seed = share_seed_csprng.next().unwrap();

            DarkpoolStateBalance::new(
                balance,
                balance_share_stream_seed,
                balance_recovery_stream_seed,
            )
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

    fn build_deposit_request_path(
        &self,
        mint: Address,
        query_params: &DepositBalanceQueryParameters,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(DEPOSIT_BALANCE_ROUTE, "account_id" => self.get_account_id(), "mint" => mint);
        let query_string =
            serde_urlencoded::to_string(query_params).map_err(RenegadeClientError::serde)?;

        Ok(format!("{}?{}", path, query_string))
    }
}
