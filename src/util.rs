//! Utility functions for the renegade-sdk
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::engine::{general_purpose as b64_general_purpose, Engine};
use hmac::Mac;
use reqwest::header::{HeaderMap, HeaderValue};

use crate::ExternalMatchClientError;

/// The header namespace to include in the HMAC
const RENEGADE_HEADER_NAMESPACE: &str = "x-renegade";
/// Header name for the HTTP auth signature; lower cased
pub const RENEGADE_AUTH_HEADER_NAME: &str = "x-renegade-auth";
/// Header name for the expiration timestamp of a signature; lower cased
pub const RENEGADE_SIG_EXPIRATION_HEADER_NAME: &str = "x-renegade-auth-expiration";

// ----------------
// | Auth Helpers |
// ----------------

/// Type alias for the hmac core implementation
type HmacSha256 = hmac::Hmac<sha2::Sha256>;
/// A 32 byte HMAC key
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct HmacKey([u8; 32]);
impl HmacKey {
    /// Create a new HMAC key from a base64 encoded string
    pub fn from_base64_string(s: &str) -> Result<Self, ExternalMatchClientError> {
        let bytes = b64_general_purpose::STANDARD
            .decode(s)
            .map_err(|_| ExternalMatchClientError::InvalidApiSecret)?;
        Ok(Self(bytes.try_into().unwrap()))
    }
}

/// Add an auth expiration and signature to a set of headers
pub fn add_expiring_auth_to_headers(
    path: &str,
    headers: &mut HeaderMap,
    body: &[u8],
    key: &HmacKey,
    expiration: Duration,
) {
    // Add a timestamp
    let expiration_ts = get_current_time_millis() + expiration.as_millis() as u64;
    headers.insert(RENEGADE_SIG_EXPIRATION_HEADER_NAME, expiration_ts.into());

    // Add the signature
    let sig = create_request_signature(path, headers, body, key);
    let b64_sig = b64_general_purpose::STANDARD_NO_PAD.encode(sig);
    let sig_header = HeaderValue::from_str(&b64_sig).expect("b64 encoding should not fail");
    headers.insert(RENEGADE_AUTH_HEADER_NAME, sig_header);
}

/// Create a request signature
pub fn create_request_signature(
    path: &str,
    headers: &HeaderMap,
    body: &[u8],
    key: &HmacKey,
) -> Vec<u8> {
    // Compute the expected HMAC
    let path_bytes = path.as_bytes();
    let header_bytes = get_header_bytes(headers);
    let payload = [path_bytes, &header_bytes, body].concat();

    // Create the HMAC
    let mut hmac = HmacSha256::new_from_slice(&key.0).expect("hmac can handle all slice lengths");
    hmac.update(&payload);
    hmac.finalize().into_bytes().to_vec()
}

/// Get the header bytes to validate in an HMAC
fn get_header_bytes(headers: &HeaderMap) -> Vec<u8> {
    let mut headers_buf = Vec::new();

    // Filter out non-Renegade headers and the auth header
    let mut renegade_headers: Vec<(String, &HeaderValue)> = headers
        .iter()
        .filter_map(|(k, v)| {
            let key = k.to_string().to_lowercase();
            if key.starts_with(RENEGADE_HEADER_NAMESPACE) && key != RENEGADE_AUTH_HEADER_NAME {
                Some((key, v))
            } else {
                None
            }
        })
        .collect();

    // Sort alphabetically, then add to the buffer
    renegade_headers.sort_by(|a, b| a.0.cmp(&b.0));
    for (key, value) in renegade_headers {
        headers_buf.extend_from_slice(key.as_bytes());
        headers_buf.extend_from_slice(value.as_bytes());
    }

    headers_buf
}

/// Returns the current unix timestamp in milliseconds, represented as u64
pub fn get_current_time_millis() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("negative timestamp").as_millis() as u64
}
