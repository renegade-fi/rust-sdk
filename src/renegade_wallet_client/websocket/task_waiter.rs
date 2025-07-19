//! A task waiter is a structure that waits for a task to complete then
//! transforms the status into a result

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::ready;
use renegade_common::types::tasks::TaskIdentifier;

use crate::{websocket::TaskNotificationRx, RenegadeClientError};

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
}

impl TaskWaiter {
    /// Create a new task waiter
    pub fn new(notification_rx: TaskNotificationRx) -> Self {
        Self { notification_rx }
    }
}

impl Future for TaskWaiter {
    type Output = Result<(), RenegadeClientError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let status = ready!(self.get_mut().notification_rx.poll_recv(cx));
        let res = status.ok_or(RenegadeClientError::custom("Task waiter closed"))?.into_result();
        Poll::Ready(res)
    }
}
