//! Shared utilities for the Renegade wallet client

use alloy::{primitives::keccak256, signers::SignerSync, signers::local::PrivateKeySigner};
use ark_ff::PrimeField;
use renegade_circuit_types::schnorr::SchnorrPrivateKey;
use renegade_constants::{EmbeddedScalarField, Scalar};
use renegade_types_core::HmacKey;
use uuid::Uuid;

// -------------
// | Constants |
// -------------

/// The message prefix used to derive the account ID
const ACCOUNT_ID_MESSAGE_PREFIX: &[u8] = b"account id";
/// The message prefix used to derive the master view seed
const MASTER_VIEW_SEED_MESSAGE_PREFIX: &[u8] = b"master view seed";
/// The message prefix used to derive the schnorr key
const SCHNORR_KEY_MESSAGE_PREFIX: &[u8] = b"schnorr key";
/// The message prefix used to derive the auth HMAC key
const AUTH_HMAC_KEY_MESSAGE_PREFIX: &[u8] = b"auth hmac key";

/// The number of bytes from a keccak hash
const KECCAK_HASH_BYTES: usize = 32;
/// The number of bytes we extend into to get a scalar
const EXTENDED_BYTES: usize = 64;
/// The number of bytes in an account ID
const ACCOUNT_ID_BYTES: usize = 16;

// ----------
// | Macros |
// ----------

/// Macro to unwrap a required field from a config, returning an error if not
/// set
macro_rules! unwrap_field {
    ($config:expr, $field:ident) => {
        $config.$field.ok_or_else(|| {
            $crate::RenegadeClientError::invalid_order(format!(
                "{} is required for order",
                stringify!($field)
            ))
        })?
    };
}
pub(crate) use unwrap_field;

// -----------
// | Helpers |
// -----------

/// Derive an account ID from a signing key & chain ID
pub fn derive_account_id(key: &PrivateKeySigner, chain_id: u64) -> Result<Uuid, String> {
    let sig_bytes = get_sig_bytes(&account_id_message(chain_id), key)?;
    let account_id = Uuid::from_slice(&sig_bytes[..ACCOUNT_ID_BYTES])
        .map_err(|e| format!("failed to derive account ID: {e}"))?;
    Ok(account_id)
}

/// Derive a master view seed from a signing key & chain ID
pub fn derive_master_view_seed(key: &PrivateKeySigner, chain_id: u64) -> Result<Scalar, String> {
    let sig_bytes = get_extended_sig_bytes(&master_view_seed_message(chain_id), key)?;
    Ok(Scalar::from_be_bytes_mod_order(&sig_bytes))
}

/// Derive a Schnorr private key from a signing key & chain ID
pub fn derive_schnorr_key(
    key: &PrivateKeySigner,
    chain_id: u64,
) -> Result<SchnorrPrivateKey, String> {
    let sig_bytes = get_extended_sig_bytes(&schnorr_key_message(chain_id), key)?;
    let inner = EmbeddedScalarField::from_be_bytes_mod_order(&sig_bytes);
    Ok(SchnorrPrivateKey { inner })
}

/// Derive an HMAC key for API auth from a signing key & chain ID
pub fn derive_auth_hmac_key(key: &PrivateKeySigner, chain_id: u64) -> Result<HmacKey, String> {
    get_sig_bytes(&auth_hmac_key_message(chain_id), key).map(HmacKey)
}

/// Generate the message to sign for account ID derivation
fn account_id_message(chain_id: u64) -> Vec<u8> {
    let mut message = Vec::from(ACCOUNT_ID_MESSAGE_PREFIX);
    message.extend_from_slice(&chain_id.to_be_bytes());
    message
}

/// Generate the message to sign for master view seed derivation
fn master_view_seed_message(chain_id: u64) -> Vec<u8> {
    let mut message = Vec::from(MASTER_VIEW_SEED_MESSAGE_PREFIX);
    message.extend_from_slice(&chain_id.to_be_bytes());
    message
}

/// Generate the message to sign for Schnorr key derivation
fn schnorr_key_message(chain_id: u64) -> Vec<u8> {
    let mut message = Vec::from(SCHNORR_KEY_MESSAGE_PREFIX);
    message.extend_from_slice(&chain_id.to_be_bytes());
    message
}

/// Generate the message to sign for auth HMAC key derivation
fn auth_hmac_key_message(chain_id: u64) -> Vec<u8> {
    let mut message = Vec::from(AUTH_HMAC_KEY_MESSAGE_PREFIX);
    message.extend_from_slice(&chain_id.to_be_bytes());
    message
}

/// Sign a message, serialize the signature into bytes
fn get_sig_bytes(msg: &[u8], key: &PrivateKeySigner) -> Result<[u8; KECCAK_HASH_BYTES], String> {
    let digest = keccak256(msg);
    let sig = key.sign_hash_sync(&digest).map_err(|e| format!("failed to sign message: {e}"))?;

    // Take the keccak hash of the signature to disperse its elements
    let bytes: Vec<u8> = sig.into();
    Ok(*keccak256(bytes))
}

/// Sign a message, serialize the signature into bytes, and extend the bytes to
/// support secure reduction into a field
fn get_extended_sig_bytes(
    msg: &[u8],
    key: &PrivateKeySigner,
) -> Result<[u8; EXTENDED_BYTES], String> {
    let sig_bytes = get_sig_bytes(msg, key)?;
    Ok(extend_to_64_bytes(&sig_bytes))
}

/// Extend the given byte array to 64 bytes, double the length of the original
///
/// This is necessary to give a uniform sampling of a field that these bytes are
/// reduced into, the bitlength must be significantly larger than the field's
/// bitlength to avoid sample bias via modular reduction
fn extend_to_64_bytes(bytes: &[u8]) -> [u8; EXTENDED_BYTES] {
    let mut extended = [0; EXTENDED_BYTES];
    let top_bytes = keccak256(bytes);
    extended[..KECCAK_HASH_BYTES].copy_from_slice(bytes);
    extended[KECCAK_HASH_BYTES..].copy_from_slice(&top_bytes.0);
    extended
}
