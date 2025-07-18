//! The websocket client for listening to Renegade events
mod client;

pub use client::*;

/// A task status notification
pub enum TaskStatusNotification {
    /// A task has been completed
    Completed,
    /// A task has failed
    Failed {
        /// The error message
        error: String,
    },
}
