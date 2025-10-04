//! A task waiter is a structure that waits for a task to complete then
//! transforms the status into a result

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures_util::{future::BoxFuture, FutureExt};
use renegade_common::types::tasks::TaskIdentifier;

use crate::{websocket::RenegadeWebsocketClient, RenegadeClientError};

/// The timeout for a task to complete
const TASK_TIMEOUT: Duration = Duration::from_secs(30);

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
    task_id: Option<TaskIdentifier>,
    /// The websocket client
    ws_client: Option<RenegadeWebsocketClient>,
    /// The underlying future that waits for the task to complete
    fut: Option<TaskWaiterFuture>,
}

impl TaskWaiter {
    /// Create a new task waiter
    pub fn new(task_id: TaskIdentifier, ws_client: RenegadeWebsocketClient) -> Self {
        Self { task_id: Some(task_id), ws_client: Some(ws_client), fut: None }
    }

    /// Watch a task until it terminates as a success or failure
    async fn watch_task(
        task_id: TaskIdentifier,
        ws_client: RenegadeWebsocketClient,
    ) -> Result<(), RenegadeClientError> {
        // Register a notification channel for the task and await
        let mut notification_rx = ws_client.watch_task(task_id).await?;
        let timeout = tokio::time::timeout(TASK_TIMEOUT, notification_rx.recv());
        let notification = timeout
            .await
            .map_err(|_| RenegadeClientError::task(format!("Task {task_id} timed out")))?
            .ok_or_else(|| RenegadeClientError::task(format!("Task {task_id} waiter closed")))?;

        notification.into_result()
    }
}

impl Future for TaskWaiter {
    type Output = Result<(), RenegadeClientError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.fut.is_none() {
            let task_id = this.task_id.take().expect("Task ID not set on first poll");
            let ws_client = this.ws_client.take().expect("Websocket client not set on first poll");

            let fut = Self::watch_task(task_id, ws_client).boxed();
            this.fut = Some(fut);
        }

        this.fut.as_mut().unwrap().as_mut().poll(cx)
    }
}
