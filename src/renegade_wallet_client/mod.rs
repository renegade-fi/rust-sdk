//! The renegade wallet client manages Renegade wallet operations

pub mod actions;
pub mod client;
mod config;

/// The error type for the renegade wallet client
#[derive(Debug, thiserror::Error)]
pub enum RenegadeClientError {
    /// An error converting from an API type to an internal type
    #[error("failed to convert from API type to internal type: {0}")]
    ConversionError(String),
    /// Custom error
    #[error("error: {0}")]
    CustomError(String),
    /// An invalid order was provided
    #[error("invalid order: {0}")]
    InvalidOrder(String),
    /// An error setting up the wallet
    #[error("failed to setup wallet: {0}")]
    SetupError(String),
    /// A relayer error
    #[error("relayer error: {0}")]
    RelayerError(String),
    /// An error sending a request to the relayer
    #[error("failed to send request to relayer: {0}")]
    RequestError(String),
    /// A wallet error
    #[error("wallet error: {0}")]
    Wallet(String),
}

impl RenegadeClientError {
    /// Create a new custom error
    #[allow(clippy::needless_pass_by_value)]
    pub fn custom<T: ToString>(msg: T) -> Self {
        Self::CustomError(msg.to_string())
    }

    /// Create a new conversion error
    #[allow(clippy::needless_pass_by_value)]
    pub fn conversion<T: ToString>(msg: T) -> Self {
        Self::ConversionError(msg.to_string())
    }

    /// Create a new invalid order error
    #[allow(clippy::needless_pass_by_value)]
    pub fn invalid_order<T: ToString>(msg: T) -> Self {
        Self::InvalidOrder(msg.to_string())
    }

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

    /// Create a new wallet error
    #[allow(clippy::needless_pass_by_value)]
    pub fn wallet<T: ToString>(msg: T) -> Self {
        Self::Wallet(msg.to_string())
    }
}
