//! Deposit funds into the wallet

use alloy::signers::local::PrivateKeySigner;
use darkpool_client::conversion::address_to_biguint;
use darkpool_client::transfer_auth::{arbitrum as arbitrum_auth, base as base_auth};
use num_bigint::BigUint;
use renegade_api::http::wallet::{
    DepositBalanceRequest, DepositBalanceResponse, DEPOSIT_BALANCE_ROUTE,
};
use renegade_circuit_types::balance::Balance;
use renegade_circuit_types::transfers::{ExternalTransfer, ExternalTransferDirection};
use renegade_common::types::transfer_auth::{DepositAuth, TransferAuth};
use renegade_utils::hex::biguint_from_hex_string;

use crate::{
    actions::{construct_http_path, prepare_wallet_update},
    client::RenegadeClient,
    websocket::TaskWaiter,
    RenegadeClientError,
};

impl RenegadeClient {
    /// Deposit funds into the wallet
    pub async fn deposit(
        &self,
        token_mint: &str,
        amount: u128,
        pkey: &PrivateKeySigner,
    ) -> Result<TaskWaiter, RenegadeClientError> {
        // Add the balance to the wallet
        let mint = biguint_from_hex_string(token_mint).map_err(RenegadeClientError::conversion)?;
        let mut wallet = self.get_internal_wallet().await?;
        let bal = Balance::new_from_mint_and_amount(mint.clone(), amount);
        wallet.add_balance(bal).map_err(RenegadeClientError::wallet)?;

        // Prepare wallet update and transfer authorization
        let update_auth = prepare_wallet_update(&mut wallet)?;
        let account_addr =
            address_to_biguint(&pkey.address()).map_err(RenegadeClientError::conversion)?;
        let transfer = ExternalTransfer {
            account_addr: account_addr.clone(),
            mint: mint.clone(),
            amount,
            direction: ExternalTransferDirection::Deposit,
        };
        let transfer_auth = self.build_deposit_auth(pkey, transfer).await?;

        // Send the deposit request to the relayer
        let wallet_id = self.secrets.wallet_id;
        let route = construct_http_path!(DEPOSIT_BALANCE_ROUTE, "wallet_id" => wallet_id);
        let request = DepositBalanceRequest {
            from_addr: account_addr,
            mint,
            amount: BigUint::from(amount),
            update_auth,
            permit_nonce: transfer_auth.permit_nonce,
            permit_deadline: transfer_auth.permit_deadline,
            permit_signature: transfer_auth.permit_signature,
        };
        let response: DepositBalanceResponse = self.post_relayer(&route, request).await?;

        // Create a task waiter for the task
        let task_id = response.task_id;
        Ok(self.get_task_waiter(task_id))
    }

    /// Build a deposit permit for the connected chain
    async fn build_deposit_auth(
        &self,
        signer: &PrivateKeySigner,
        transfer: ExternalTransfer,
    ) -> Result<DepositAuth, RenegadeClientError> {
        let pk_root = self.secrets.keychain.pk_root();
        let transfer_with_auth = if self.is_solidity_chain() {
            base_auth::build_deposit_auth(
                signer,
                &pk_root,
                transfer,
                self.config.permit2_address,
                self.config.darkpool_address,
                self.config.chain_id,
            )
            .map_err(RenegadeClientError::wallet)?
        } else {
            arbitrum_auth::build_deposit_auth(
                signer,
                &pk_root,
                transfer,
                self.config.permit2_address,
                self.config.darkpool_address,
                self.config.chain_id,
            )
            .map_err(RenegadeClientError::wallet)?
        };

        match transfer_with_auth.transfer_auth {
            TransferAuth::Deposit(deposit_auth) => Ok(deposit_auth),
            TransferAuth::Withdrawal(_) => unreachable!(),
        }
    }
}
