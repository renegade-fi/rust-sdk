//! A task waiter is a structure that waits for a task to complete then
//! transforms the status into a result

use std::time::Duration;

use renegade_common::types::tasks::TaskIdentifier;

use crate::{websocket::RenegadeWebsocketClient, RenegadeClientError};

/// The timeout for a task to complete
const TASK_TIMEOUT: Duration = Duration::from_secs(30);

/// A task status notification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatusNotification {
    /// A task has been completed
    Success,
    /// A task has failed
    Failed {
        /// The error message
        error: String,
    },
}

impl TaskStatusNotification {
    /// Convert the task status into a Result<(), RenegadeClientError>
    pub fn into_result(self) -> Result<(), RenegadeClientError> {
        match self {
            Self::Success => Ok(()),
            Self::Failed { error } => Err(RenegadeClientError::task(error)),
        }
    }
}

/// A thin wrapper around a notification channel that waits for a task to
/// complete then transforms the status into a result
pub struct TaskWaiter {
    /// The task ID
    task_id: TaskIdentifier,
    /// The websocket client
    ws_client: RenegadeWebsocketClient,
}

impl TaskWaiter {
    /// Create a new task waiter
    pub fn new(task_id: TaskIdentifier, ws_client: RenegadeWebsocketClient) -> Self {
        Self { task_id, ws_client }
    }

    /// Watch a task until it terminates as a success or failure
    pub async fn watch_task(&mut self) -> Result<(), RenegadeClientError> {
        // Register a notification channel for the task and await
        let mut notification_rx = self.ws_client.watch_task(self.task_id).await?;
        let timeout = tokio::time::timeout(TASK_TIMEOUT, notification_rx.recv());
        let notification = timeout
            .await
            .map_err(|_| RenegadeClientError::task("Task timed out"))?
            .ok_or_else(|| RenegadeClientError::task("Task waiter closed"))?;

        notification.into_result()
    }
}
