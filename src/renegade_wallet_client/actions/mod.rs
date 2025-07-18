//! Actions to update a Renegade wallet

pub mod cancel_order;
pub mod create_wallet;
pub mod deposit;
pub mod get_wallet;
pub mod place_order;
pub mod withdraw;

use renegade_api::http::wallet::WalletUpdateAuthorization;
use renegade_common::types::wallet::Wallet;

use crate::RenegadeClientError;

// -----------
// | Helpers |
// -----------

/// Get wallet update authorization for a wallet after an update
///
/// Update auth is the signature of a commitment to the wallet's new state after
/// it is reblinded.
pub(crate) fn prepare_wallet_update(
    wallet: &mut Wallet,
) -> Result<WalletUpdateAuthorization, RenegadeClientError> {
    // First reblind the wallet
    wallet.reblind_wallet();

    // Sign a commitment to the wallet's new state
    let commitment = wallet.get_wallet_share_commitment();
    let sig = wallet.sign_commitment(commitment).map_err(RenegadeClientError::custom)?;
    let statement_sig = sig.as_bytes().to_vec();

    // Return the update auth
    let update_auth = WalletUpdateAuthorization { statement_sig, new_root_key: None };
    Ok(update_auth)
}

/// Constructs an HTTP path by replacing URL parameters with given values
macro_rules! construct_http_path {
    ($base_url:expr $(, $param:expr => $value:expr)*) => {{
        let mut url = $base_url.to_string();
        $(
            let placeholder = format!(":{}", $param);
            url = url.replace(&placeholder, &$value.to_string());
        )*
        url
    }};
}
pub(crate) use construct_http_path;
