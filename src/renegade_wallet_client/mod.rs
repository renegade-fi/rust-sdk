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
    /// An error subscribing/unsubscribing to a websocket topic
    #[error("websocket topic subscription error: {0}")]
    Subscription(String),
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

    /// Create a new websocket topic subscription error
    #[allow(clippy::needless_pass_by_value)]
    pub fn subscription<T: ToString>(msg: T) -> Self {
        Self::Subscription(msg.to_string())
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
