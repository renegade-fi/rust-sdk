//! Protocol authentication helpers for the darkpool client

use eyre::Result;
use renegade_api::http::wallet::WalletUpdateAuthorization;
use renegade_common::types::wallet::Wallet;

use crate::wrap_eyre;

/// Reblind a wallet and get a commitment to the new shares
///
/// Use in wallet update methods that require a statement signature
pub fn reblind_and_authorize_update(wallet: &mut Wallet) -> Result<WalletUpdateAuthorization> {
    wallet.reblind_wallet();
    let new_shares_commitment = wallet.get_wallet_share_commitment();
    let statement_sig = wrap_eyre!(wallet.sign_commitment(new_shares_commitment))?;
    Ok(WalletUpdateAuthorization { statement_sig: statement_sig.to_vec(), new_root_key: None })
}
