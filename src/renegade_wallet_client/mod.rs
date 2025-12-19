//! The renegade wallet client manages Renegade wallet operations

use crate::{http::RelayerHttpClientError, renegade_api_types::tasks::TaskIdentifier};

pub mod actions;
pub mod client;
pub mod config;
pub mod renegade_api_types;
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
    /// An invalid order update was provided
    #[error("invalid order update: {0}")]
    InvalidOrderUpdate(String),
    /// An error signing a message
    #[error("failed to sign message: {0}")]
    Signing(String),
    /// A relayer error
    #[error("relayer error: {0}")]
    Relayer(RelayerHttpClientError),
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
    #[error("task error: task {task_id}: {message}")]
    Task {
        /// The task identifier
        task_id: TaskIdentifier,
        /// The error message
        message: String,
    },
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

    /// Create a new invalid order update error
    #[allow(clippy::needless_pass_by_value)]
    pub fn invalid_order_update<T: ToString>(msg: T) -> Self {
        Self::InvalidOrderUpdate(msg.to_string())
    }

    /// Create a new signing error
    #[allow(clippy::needless_pass_by_value)]
    pub fn signing<T: ToString>(msg: T) -> Self {
        Self::Signing(msg.to_string())
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
    pub fn task<T: ToString>(task_id: TaskIdentifier, msg: T) -> Self {
        Self::Task { task_id, message: msg.to_string() }
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

impl From<RelayerHttpClientError> for RenegadeClientError {
    fn from(err: RelayerHttpClientError) -> Self {
        Self::Relayer(err)
    }
}
