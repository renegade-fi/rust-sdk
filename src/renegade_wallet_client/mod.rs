//! The renegade wallet client manages Renegade wallet operations

pub mod actions;
pub mod client;
mod config;
pub mod websocket;

/// The error type for the renegade wallet client
#[derive(Debug, thiserror::Error)]
pub enum RenegadeClientError {
    /// An error converting from an API type to an internal type
    #[error("failed to convert from API type to internal type: {0}")]
    Conversion(String),
    /// Custom error
    #[error("error: {0}")]
    Custom(String),
    /// An invalid order was provided
    #[error("invalid order: {0}")]
    InvalidOrder(String),
    /// A relayer error
    #[error("relayer error: {0}")]
    Relayer(String),
    /// An error sending a request to the relayer
    #[error("failed to send request to relayer: {0}")]
    Request(String),
    /// A serde error
    #[error("serde error: {0}")]
    Serde(String),
    /// An error setting up the wallet
    #[error("failed to setup wallet: {0}")]
    Setup(String),
    /// A task error
    #[error("task error: {0}")]
    Task(String),
    /// A wallet error
    #[error("wallet error: {0}")]
    Wallet(String),
    /// A websocket error
    #[error("websocket error: {0}")]
    Websocket(String),
}

impl RenegadeClientError {
    /// Create a new custom error
    #[allow(clippy::needless_pass_by_value)]
    pub fn custom<T: ToString>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }

    /// Create a new conversion error
    #[allow(clippy::needless_pass_by_value)]
    pub fn conversion<T: ToString>(msg: T) -> Self {
        Self::Conversion(msg.to_string())
    }

    /// Create a new invalid order error
    #[allow(clippy::needless_pass_by_value)]
    pub fn invalid_order<T: ToString>(msg: T) -> Self {
        Self::InvalidOrder(msg.to_string())
    }

    /// Create a new setup error
    #[allow(clippy::needless_pass_by_value)]
    pub fn setup<T: ToString>(msg: T) -> Self {
        Self::Setup(msg.to_string())
    }

    /// Create a new serde error
    #[allow(clippy::needless_pass_by_value)]
    pub fn serde<T: ToString>(msg: T) -> Self {
        Self::Serde(msg.to_string())
    }

    /// Create a new task error
    #[allow(clippy::needless_pass_by_value)]
    pub fn task<T: ToString>(msg: T) -> Self {
        Self::Task(msg.to_string())
    }

    /// Create a new relayer error
    #[allow(clippy::needless_pass_by_value)]
    pub fn relayer<T: ToString>(msg: T) -> Self {
        Self::Relayer(msg.to_string())
    }

    /// Create a new request error
    #[allow(clippy::needless_pass_by_value)]
    pub fn request<T: ToString>(msg: T) -> Self {
        Self::Request(msg.to_string())
    }

    /// Create a new wallet error
    #[allow(clippy::needless_pass_by_value)]
    pub fn wallet<T: ToString>(msg: T) -> Self {
        Self::Wallet(msg.to_string())
    }

    /// Create a new websocket error
    #[allow(clippy::needless_pass_by_value)]
    pub fn websocket<T: ToString>(msg: T) -> Self {
        Self::Websocket(msg.to_string())
    }
}
