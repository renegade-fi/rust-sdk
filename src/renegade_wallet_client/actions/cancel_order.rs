//! Cancels an order in the wallet

use alloy::primitives::U256;
use renegade_darkpool_types::intent::DarkpoolStateIntent;
use renegade_external_api::{
    http::order::{CANCEL_ORDER_ROUTE, CancelOrderRequest, CancelOrderResponse},
    types::{ApiOrder, ApiPublicIntentPermit, OrderAuth, SignatureWithNonce},
};
use renegade_solidity_abi::v2::IDarkpoolV2::{
    PublicIntentPermit, SignatureWithNonce as SolSignatureWithNonce,
};
use uuid::Uuid;

use crate::{
    RenegadeClientError,
    actions::{NON_BLOCKING_PARAM, construct_http_path},
    client::RenegadeClient,
    websocket::{DEFAULT_TASK_TIMEOUT, TaskWaiter},
};

/// The cancel domain separator, matching the contract's `CANCEL_DOMAIN`
const CANCEL_DOMAIN: &[u8] = b"cancel";

// --- Public Actions --- //
impl RenegadeClient {
    /// Cancels the order with the given ID. Waits for the order cancellation
    /// task to complete before returning.
    pub async fn cancel_order(&self, order_id: Uuid) -> Result<(), RenegadeClientError> {
        let request = self.build_cancel_order_request(order_id).await?;

        let path = self.build_cancel_order_request_path(order_id, false)?;

        self.relayer_client.post::<_, CancelOrderResponse>(&path, request).await?;

        Ok(())
    }

    /// Enqueues an order cancellation task in the relayer. Returns a
    /// `TaskWaiter` that can be used to await task completion.
    pub async fn enqueue_order_cancellation(
        &self,
        order_id: Uuid,
    ) -> Result<TaskWaiter, RenegadeClientError> {
        let request = self.build_cancel_order_request(order_id).await?;

        let path = self.build_cancel_order_request_path(order_id, true)?;

        let CancelOrderResponse { task_id, .. } = self.relayer_client.post(&path, request).await?;

        // Create a task waiter for the task
        let task_waiter = self.watch_task(task_id, DEFAULT_TASK_TIMEOUT).await?;

        Ok(task_waiter)
    }
}

// --- Private Helpers --- //
impl RenegadeClient {
    /// Builds the order cancellation request from the given order ID
    async fn build_cancel_order_request(
        &self,
        order_id: Uuid,
    ) -> Result<CancelOrderRequest, RenegadeClientError> {
        let (order, auth) = self.get_order_with_auth(order_id).await?;

        let cancel_signature = if let OrderAuth::PublicOrder { permit, intent_signature } = auth {
            self.build_ring0_cancel_signature(permit, intent_signature)?
        } else {
            self.build_private_cancel_signature(order)?
        };

        Ok(CancelOrderRequest { cancel_signature })
    }

    /// Builds the request path for the cancel order endpoint
    fn build_cancel_order_request_path(
        &self,
        order_id: Uuid,
        non_blocking: bool,
    ) -> Result<String, RenegadeClientError> {
        let path = construct_http_path!(CANCEL_ORDER_ROUTE, "account_id" => self.get_account_id(), "order_id" => order_id);
        let query_string =
            serde_urlencoded::to_string(&[(NON_BLOCKING_PARAM, non_blocking.to_string())])
                .map_err(RenegadeClientError::serde)?;

        Ok(format!("{path}?{query_string}"))
    }

    // --- Signing Helpers --- //

    /// Build the cancel signature for a ring0 (public) order
    ///
    /// The cancel digest is: sign over `"cancel" || intentNullifier` where
    /// `intentNullifier = keccak256(abi.encode(permit) ||
    /// originalIntentNonce)`.
    fn build_ring0_cancel_signature(
        &self,
        permit: ApiPublicIntentPermit,
        intent_signature: SignatureWithNonce,
    ) -> Result<SignatureWithNonce, RenegadeClientError> {
        let chain_id = self.get_chain_id();
        let signer = self.get_account_signer();

        // Convert API types to solidity-abi types
        let permit: PublicIntentPermit = permit.into();
        let sol_intent_sig: SolSignatureWithNonce = intent_signature.into();

        // Compute intent nullifier: H(intentHash || originalNonce)
        let intent_nullifier = permit.compute_nullifier(sol_intent_sig.nonce);

        // Build cancel payload: CANCEL_DOMAIN || intentNullifier
        let nullifier_bytes = intent_nullifier.to_be_bytes::<{ U256::BYTES }>();
        let cancel_payload = [CANCEL_DOMAIN, nullifier_bytes.as_slice()].concat();

        // Sign: H(H(cancel_payload) || nonce || chainId)
        let sol_sig = SolSignatureWithNonce::sign(&cancel_payload, chain_id, signer)
            .map_err(RenegadeClientError::signing)?;

        Ok(sol_sig.into())
    }

    /// Build the cancel signature for a ring1+ (private) order
    ///
    /// Ring1+ cancel: sign the Poseidon nullifier bytes, properly incorporating
    /// nonce + chainId via `SignatureWithNonce::sign`.
    fn build_private_cancel_signature(
        &self,
        order: ApiOrder,
    ) -> Result<SignatureWithNonce, RenegadeClientError> {
        let chain_id = self.get_chain_id();
        let signer = self.get_account_signer();

        let intent: DarkpoolStateIntent = order.into();
        let nullifier = intent.compute_nullifier();
        let nullifier_bytes = nullifier.to_bytes_be();

        let sol_sig = SolSignatureWithNonce::sign(&nullifier_bytes, chain_id, signer)
            .map_err(RenegadeClientError::signing)?;

        Ok(sol_sig.into())
    }
}
