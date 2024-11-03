//! The error type for the external match client

/// An error that can occur when requesting an external match
#[derive(Debug, thiserror::Error)]
pub enum ExternalMatchClientError {
    /// An error that can occur when requesting an external match
    #[error("an error that can occur when requesting an external match")]
    Http(#[from] reqwest::Error),
    /// An error indicating that the api key is invalid
    #[error("the api key is invalid")]
    InvalidApiKey,
    /// An error indicating that the api secret is invalid
    #[error("the api secret is invalid")]
    InvalidApiSecret,
    /// An error indicating that an order is invalid
    #[error("invalid order: {0}")]
    InvalidOrder(String),
}

impl ExternalMatchClientError {
    /// Construct a new invalid order error
    #[allow(clippy::needless_pass_by_value)]
    pub fn invalid_order<T: ToString>(msg: T) -> Self {
        Self::InvalidOrder(msg.to_string())
    }
}
