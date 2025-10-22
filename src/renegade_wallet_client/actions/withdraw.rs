//! Withdraw funds from the wallet

use std::time::Duration;

use alloy::signers::local::PrivateKeySigner;
use darkpool_client::{
    conversion::address_to_biguint,
    transfer_auth::{arbitrum as arbitrum_auth, base as base_auth},
};
use k256::ecdsa::SigningKey;
use num_bigint::BigUint;
use renegade_api::{
    http::wallet::{
        PayFeesResponse, WithdrawBalanceRequest, WithdrawBalanceResponse, PAY_FEES_ROUTE,
        WITHDRAW_BALANCE_ROUTE,
    },
    EmptyRequestResponse,
};
use renegade_circuit_types::transfers::{ExternalTransfer, ExternalTransferDirection};
use renegade_common::types::transfer_auth::{TransferAuth, WithdrawalAuth};
use renegade_utils::hex::biguint_from_hex_string;
use tracing::{error, info};

use crate::{
    actions::{construct_http_path, prepare_wallet_update},
    client::RenegadeClient,
    websocket::TaskWaiter,
    RenegadeClientError,
};

/// The timeout for a withdrawal action to complete.
///
/// This is longer than the default since any enqueued fee payment tasks must
/// complete first.
const TASK_WAITER_TIMEOUT: Duration = Duration::from_secs(120);

impl RenegadeClient {
    /// Withdraw funds from the wallet
    pub async fn withdraw(
        &self,
        token_mint: &str,
        amount: u128,
        pkey: &PrivateKeySigner,
    ) -> Result<TaskWaiter, RenegadeClientError> {
        // First, pay all fees on the wallet
        self.enqueue_pay_fees().await?;

        // Remove the balance from the wallet
        let mint = biguint_from_hex_string(token_mint).map_err(RenegadeClientError::conversion)?;

        let mut wallet = self.get_internal_wallet().await?;
        let back_of_queue_wallet = wallet.clone();

        wallet.withdraw(&mint, amount).map_err(RenegadeClientError::wallet)?;

        // Prepare wallet update and transfer authorization
        let update_auth = prepare_wallet_update(&mut wallet)?;
        let account_addr =
            address_to_biguint(&pkey.address()).map_err(RenegadeClientError::conversion)?;

        let transfer = ExternalTransfer {
            account_addr: account_addr.clone(),
            mint: mint.clone(),
            amount,
            direction: ExternalTransferDirection::Withdrawal,
        };
        let transfer_auth = self.build_withdraw_auth(transfer)?;

        // Send the withdrawal request to the relayer
        let wallet_id = self.secrets.wallet_id;
        let route = construct_http_path!(WITHDRAW_BALANCE_ROUTE, "wallet_id" => wallet_id, "mint" => token_mint);
        let request = WithdrawBalanceRequest {
            destination_addr: account_addr,
            amount: BigUint::from(amount),
            update_auth,
            external_transfer_sig: transfer_auth.external_transfer_signature,
        };
        let result: Result<WithdrawBalanceResponse, _> =
            self.relayer_client.post(&route, request).await;

        if let Err(e) = result {
            let balances = back_of_queue_wallet
                .balances
                .iter()
                .map(|(_, b)| serde_json::to_string_pretty(b))
                .collect::<Result<Vec<String>, _>>()
                .map_err(RenegadeClientError::serde)?;

            let orders = back_of_queue_wallet
                .orders
                .iter()
                .map(|(_, o)| serde_json::to_string_pretty(o))
                .collect::<Result<Vec<String>, _>>()
                .map_err(RenegadeClientError::serde)?;

            error!(
                balances = ?balances,
                orders = ?orders,
                "Failed to withdraw from wallet: {e}"
            );

            return Err(e.into());
        }

        let response = result.unwrap();

        // Create a task waiter for the task
        let task_id = response.task_id;
        let task_waiter_builder = self.get_task_waiter_builder(task_id);
        Ok(task_waiter_builder.with_timeout(TASK_WAITER_TIMEOUT).build())
    }

    /// Enqueue a task to pay fees on the wallet
    async fn enqueue_pay_fees(&self) -> Result<(), RenegadeClientError> {
        let wallet_id = self.secrets.wallet_id;
        let route = construct_http_path!(PAY_FEES_ROUTE, "wallet_id" => wallet_id);
        let response: PayFeesResponse =
            self.relayer_client.post(&route, EmptyRequestResponse {}).await?;

        let task_id_strings: Vec<String> =
            response.task_ids.iter().map(|id| id.to_string()).collect();

        info!("Enqueued fee payment tasks: {}", task_id_strings.join(", "));

        Ok(())
    }

    /// Build a withdraw permit for the connected chain
    fn build_withdraw_auth(
        &self,
        transfer: ExternalTransfer,
    ) -> Result<WithdrawalAuth, RenegadeClientError> {
        // Pull the root key from the keychain stored locally
        let root_key = &self
            .secrets
            .keychain
            .sk_root()
            .ok_or_else(|| RenegadeClientError::wallet("No root key found in keychain"))?;
        let signing_key: SigningKey = root_key.try_into().map_err(|_| {
            RenegadeClientError::wallet("Failed to convert root key to signing key")
        })?;

        // Build the withdrawal auth
        let transfer_with_auth = if self.is_solidity_chain() {
            base_auth::build_withdrawal_auth(&signing_key.into(), transfer)
                .map_err(RenegadeClientError::wallet)?
        } else {
            arbitrum_auth::build_withdrawal_auth(&signing_key.into(), transfer)
                .map_err(RenegadeClientError::wallet)?
        };

        match transfer_with_auth.transfer_auth {
            TransferAuth::Withdrawal(withdrawal_auth) => Ok(withdrawal_auth),
            TransferAuth::Deposit(_) => unreachable!(),
        }
    }
}
