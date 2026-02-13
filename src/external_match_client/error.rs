//! The error type for the external match client

use reqwest::StatusCode;

use crate::http::RelayerHttpClientError;

/// An error that can occur when requesting an external match
#[derive(Debug, thiserror::Error)]
pub enum ExternalMatchClientError {
    /// An error that can occur when requesting an external match
    #[error(
        "error while requesting external match: status={}, message={1}",
        .0.as_ref().map(ToString::to_string).unwrap_or_else(|| "none".to_string())
    )]
    Http(Option<StatusCode>, String),
    /// An error indicating that the api key is invalid
    #[error("the api key is invalid")]
    InvalidApiKey,
    /// An error indicating that the api secret is invalid
    #[error("the api secret is invalid")]
    InvalidApiSecret,
    /// An invalid modification to a malleable match
    #[error("invalid modification to a malleable match: {0}")]
    InvalidModification(String),
    /// An error indicating that an order is invalid
    #[error("invalid order: {0}")]
    InvalidOrder(String),
    /// An error deserializing a response
    #[error("error deserializing a response: {0}")]
    Deserialize(String),
}

impl ExternalMatchClientError {
    /// Construct a new http error
    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn http<T: ToString>(status: StatusCode, msg: T) -> Self {
        Self::Http(Some(status), msg.to_string())
    }

    /// Construct a new invalid modification error
    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn invalid_modification<T: ToString>(msg: T) -> Self {
        Self::InvalidModification(msg.to_string())
    }

    /// Construct a new invalid order error
    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn invalid_order<T: ToString>(msg: T) -> Self {
        Self::InvalidOrder(msg.to_string())
    }

    /// Construct a new deserialize error
    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn deserialize<T: ToString>(msg: T) -> Self {
        Self::Deserialize(msg.to_string())
    }
}

impl From<reqwest::Error> for ExternalMatchClientError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(None, err.to_string())
    }
}

impl From<RelayerHttpClientError> for ExternalMatchClientError {
    fn from(err: RelayerHttpClientError) -> Self {
        Self::Http(None, err.to_string())
    }
}
