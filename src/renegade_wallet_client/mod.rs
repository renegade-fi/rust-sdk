//! The renegade wallet client manages Renegade wallet operations

pub mod client;

/// The error type for the renegade wallet client
#[derive(Debug, thiserror::Error)]
pub enum RenegadeClientError {
    /// An error setting up the wallet
    #[error("failed to setup wallet: {0}")]
    SetupError(String),
}

impl RenegadeClientError {
    /// Create a new setup error
    #[allow(clippy::needless_pass_by_value)]
    pub fn setup_error<T: ToString>(msg: T) -> Self {
        Self::SetupError(msg.to_string())
    }
}
