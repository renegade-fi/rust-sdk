//! Types and utilities for HMAC-based authentication
//!
//! Inlines `HmacKey` (from `types-core`) and `add_expiring_auth_to_headers`
//! (from `external-api`) so the `external-match-client` feature path does not
//! pull in the full renegade dependency tree.

use base64::engine::{Engine, general_purpose as b64_general_purpose};
use hmac::Mac;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// -------------
// | Constants |
// -------------

/// The length of an HMAC key in bytes
pub const HMAC_KEY_LEN: usize = 32;

/// The header name for the renegade auth signature
const RENEGADE_AUTH_HEADER_NAME: &str = "x-renegade-auth";

/// The header name for the renegade auth signature expiration
const RENEGADE_SIG_EXPIRATION_HEADER_NAME: &str = "x-renegade-auth-expiration";

/// The header namespace to include in the HMAC
const RENEGADE_HEADER_NAMESPACE: &str = "x-renegade";

// ---------
// | Types |
// ---------

/// Type alias for the hmac core implementation
type HmacSha256 = hmac::Hmac<Sha256>;

/// A type representing a symmetric HMAC key
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HmacKey(pub [u8; HMAC_KEY_LEN]);

#[cfg(feature = "darkpool-client")]
impl From<HmacKey> for renegade_types_core::HmacKey {
    fn from(key: HmacKey) -> Self {
        Self(key.0)
    }
}

#[cfg(feature = "darkpool-client")]
impl From<renegade_types_core::HmacKey> for HmacKey {
    fn from(key: renegade_types_core::HmacKey) -> Self {
        Self(key.0)
    }
}

impl HmacKey {
    /// Create a new HMAC key from a hex string
    pub fn new(hex: &str) -> Result<Self, String> {
        Self::from_hex_string(hex)
    }

    /// Get the inner bytes
    pub fn inner(&self) -> &[u8; HMAC_KEY_LEN] {
        &self.0
    }

    /// Create a new random HMAC key
    #[cfg(feature = "darkpool-client")]
    pub fn random() -> Self {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = [0; HMAC_KEY_LEN];
        rng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Convert the HMAC key to a hex string
    pub fn to_hex_string(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }

    /// Try to convert a hex string to an HMAC key
    pub fn from_hex_string(hex_str: &str) -> Result<Self, String> {
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let bytes = hex::decode(hex_str)
            .map_err(|e| format!("error deserializing bytes from hex string: {e}"))?;

        if bytes.len() != HMAC_KEY_LEN {
            return Err(format!("expected {HMAC_KEY_LEN} byte HMAC key, got {}", bytes.len()));
        }

        Ok(Self(bytes.try_into().unwrap()))
    }

    /// Convert the HMAC key to a base64 string
    pub fn to_base64_string(&self) -> String {
        b64_general_purpose::STANDARD.encode(self.0)
    }

    /// Try to convert a base64 string to an HMAC key
    pub fn from_base64_string(base64: &str) -> Result<Self, String> {
        let bytes = b64_general_purpose::STANDARD.decode(base64).map_err(|e| e.to_string())?;
        if bytes.len() != HMAC_KEY_LEN {
            return Err(format!("expected {HMAC_KEY_LEN} byte HMAC key, got {}", bytes.len()));
        }

        Ok(Self(bytes.try_into().unwrap()))
    }

    /// Try to create an HMAC key from a byte slice
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != HMAC_KEY_LEN {
            return Err(format!("expected {HMAC_KEY_LEN} byte HMAC key, got {}", bytes.len()));
        }

        Ok(Self(bytes.try_into().unwrap()))
    }

    /// Compute the HMAC of a message
    pub fn compute_mac(&self, msg: &[u8]) -> Vec<u8> {
        let mut hmac =
            HmacSha256::new_from_slice(self.inner()).expect("hmac can handle all slice lengths");
        hmac.update(msg);
        hmac.finalize().into_bytes().to_vec()
    }

    /// Verify the HMAC of a message
    pub fn verify_mac(&self, msg: &[u8], mac: &[u8]) -> bool {
        self.compute_mac(msg) == mac
    }
}

// --------------------
// | Public Interface |
// --------------------

/// Add an auth expiration and signature to a set of headers
pub fn add_expiring_auth_to_headers(
    path: &str,
    headers: &mut HeaderMap,
    body: &[u8],
    key: &HmacKey,
    expiration: Duration,
) {
    // Add a timestamp
    let now_millis =
        SystemTime::now().duration_since(UNIX_EPOCH).expect("negative timestamp").as_millis()
            as u64;
    let expiration_ts = now_millis + expiration.as_millis() as u64;
    headers.insert(RENEGADE_SIG_EXPIRATION_HEADER_NAME, expiration_ts.into());

    // Add the signature
    let sig = create_request_signature(path, headers, body, key);
    let b64_sig = b64_general_purpose::STANDARD_NO_PAD.encode(sig);
    let sig_header = HeaderValue::from_str(&b64_sig).expect("b64 encoding should not fail");
    headers.insert(RENEGADE_AUTH_HEADER_NAME, sig_header);
}

// -----------
// | Helpers |
// -----------

/// Create a request signature
fn create_request_signature(
    path: &str,
    headers: &HeaderMap,
    body: &[u8],
    key: &HmacKey,
) -> Vec<u8> {
    let path_bytes = path.as_bytes();
    let header_bytes = get_header_bytes(headers);
    let payload = [path_bytes, &header_bytes, body].concat();

    key.compute_mac(&payload)
}

/// Get the header bytes to include in an HMAC
fn get_header_bytes(headers: &HeaderMap) -> Vec<u8> {
    let mut headers_buf = Vec::new();

    // Filter out non-Renegade headers and the auth header
    let mut renegade_headers = headers
        .iter()
        .filter_map(|(k, v)| {
            let key = k.to_string().to_lowercase();
            if key.starts_with(RENEGADE_HEADER_NAMESPACE) && key != RENEGADE_AUTH_HEADER_NAME {
                Some((key, v))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    // Sort alphabetically, then add to the buffer
    renegade_headers.sort_by(|a, b| a.0.cmp(&b.0));
    for (key, value) in renegade_headers {
        headers_buf.extend_from_slice(key.as_bytes());
        headers_buf.extend_from_slice(value.as_bytes());
    }

    headers_buf
}
