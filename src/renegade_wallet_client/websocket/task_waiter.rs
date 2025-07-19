//! A task waiter is a structure that waits for a task to complete then
//! transforms the status into a result

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Sleep as SleepFuture},
};

use crate::{websocket::TaskNotificationRx, RenegadeClientError};

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
    /// The notification channel
    notification_rx: TaskNotificationRx,
    /// The timeout future
    timeout: Pin<Box<SleepFuture>>,
}

impl TaskWaiter {
    /// Create a new task waiter
    pub fn new(notification_rx: TaskNotificationRx) -> Self {
        Self { notification_rx, timeout: Box::pin(tokio::time::sleep(TASK_TIMEOUT)) }
    }
}

impl Future for TaskWaiter {
    type Output = Result<(), RenegadeClientError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // First, try to poll the notification receiver
        match self.notification_rx.poll_recv(cx) {
            Poll::Ready(Some(notification)) => {
                return Poll::Ready(notification.into_result());
            },
            Poll::Ready(None) => {
                return Poll::Ready(Err(RenegadeClientError::task("Task waiter closed")));
            },
            Poll::Pending => {
                // Continue to check timeout
            },
        }

        // Check if the timeout has elapsed
        match self.timeout.as_mut().poll(cx) {
            Poll::Ready(()) => Poll::Ready(Err(RenegadeClientError::task("Task timed out"))),
            Poll::Pending => Poll::Pending,
        }
    }
}
