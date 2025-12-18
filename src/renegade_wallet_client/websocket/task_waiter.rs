//! A task waiter is a structure that waits for a task to complete then
//! transforms the status into a result

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures_util::{future::BoxFuture, FutureExt};

use crate::{
    renegade_api_types::TaskIdentifier, websocket::RenegadeWebsocketClient, RenegadeClientError,
};

/// The timeout for a task to complete
const DEFAULT_TASK_TIMEOUT: Duration = Duration::from_secs(60);

/// The future type for a task waiter
type TaskWaiterFuture = BoxFuture<'static, Result<(), RenegadeClientError>>;

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
    pub fn into_result(self, task_id: TaskIdentifier) -> Result<(), RenegadeClientError> {
        match self {
            Self::Success => Ok(()),
            Self::Failed { error } => Err(RenegadeClientError::task(task_id, error)),
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
    /// The duration to wait for the task to complete before timing out
    timeout: Duration,
    /// The underlying future that waits for the task to complete
    fut: Option<TaskWaiterFuture>,
}

impl TaskWaiter {
    /// Create a new task waiter
    pub fn new(
        task_id: TaskIdentifier,
        ws_client: RenegadeWebsocketClient,
        timeout: Duration,
    ) -> Self {
        Self { task_id, ws_client, timeout, fut: None }
    }

    /// Watch a task until it terminates as a success or failure
    async fn watch_task(
        task_id: TaskIdentifier,
        ws_client: RenegadeWebsocketClient,
        timeout: Duration,
    ) -> Result<(), RenegadeClientError> {
        // Register a notification channel for the task and await
        let mut notification_rx = ws_client.watch_task(task_id).await?;
        let timeout = tokio::time::timeout(timeout, notification_rx.recv());
        let notification = timeout
            .await
            .map_err(|_| RenegadeClientError::task(task_id, "Task timed out"))?
            .ok_or_else(|| RenegadeClientError::task(task_id, "Task waiter closed"))?;

        notification.into_result(task_id)
    }
}

impl Future for TaskWaiter {
    type Output = Result<(), RenegadeClientError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.fut.is_none() {
            let fut = Self::watch_task(this.task_id, this.ws_client.clone(), this.timeout).boxed();
            this.fut = Some(fut);
        }

        this.fut.as_mut().unwrap().as_mut().poll(cx)
    }
}

/// A builder for creating task waiters
pub struct TaskWaiterBuilder {
    /// The task ID
    task_id: TaskIdentifier,
    /// The websocket client
    ws_client: RenegadeWebsocketClient,
    /// The duration to wait for the task to complete before timing out
    timeout: Option<Duration>,
}

impl TaskWaiterBuilder {
    /// Create a new task waiter builder
    pub fn new(task_id: TaskIdentifier, ws_client: RenegadeWebsocketClient) -> Self {
        Self { task_id, ws_client, timeout: None }
    }

    /// Set the timeout for the task waiter
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Build the task waiter
    pub fn build(self) -> TaskWaiter {
        TaskWaiter::new(self.task_id, self.ws_client, self.timeout.unwrap_or(DEFAULT_TASK_TIMEOUT))
    }
}
