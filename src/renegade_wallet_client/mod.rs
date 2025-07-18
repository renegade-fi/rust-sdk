//! The renegade wallet client manages Renegade wallet operations

pub mod actions;
pub mod client;

/// The error type for the renegade wallet client
#[derive(Debug, thiserror::Error)]
pub enum RenegadeClientError {
    /// An error setting up the wallet
    #[error("failed to setup wallet: {0}")]
    SetupError(String),
    /// A relayer error
    #[error("relayer error: {0}")]
    RelayerError(String),
    /// An error sending a request to the relayer
    #[error("failed to send request to relayer: {0}")]
    RequestError(String),
}

impl RenegadeClientError {
    /// Create a new setup error
    #[allow(clippy::needless_pass_by_value)]
    pub fn setup<T: ToString>(msg: T) -> Self {
        Self::SetupError(msg.to_string())
    }

    /// Create a new relayer error
    #[allow(clippy::needless_pass_by_value)]
    pub fn relayer<T: ToString>(msg: T) -> Self {
        Self::RelayerError(msg.to_string())
    }

    /// Create a new request error
    #[allow(clippy::needless_pass_by_value)]
    pub fn request<T: ToString>(msg: T) -> Self {
        Self::RequestError(msg.to_string())
    }
}
