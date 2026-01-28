//! A task waiter is a structure that waits for a task to complete then
//! transforms the status into a result

use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use futures_util::{FutureExt, Stream, future::BoxFuture};
use tokio::sync::{
    RwLock,
    oneshot::{self, Receiver as OneshotReceiver, Sender as OneshotSender},
};
use tokio_stream::StreamExt;
use tracing::error;

use crate::{
    RenegadeClientError,
    renegade_api_types::{
        tasks::{ApiTask, TaskIdentifier},
        websocket::TaskUpdateWebsocketMessage,
    },
};

// -------------
// | Constants |
// -------------

/// The timeout for a task to complete
pub const DEFAULT_TASK_TIMEOUT: Duration = Duration::from_secs(60);

// ----------------
// | Type Aliases |
// ----------------

/// A oneshot channel on which to send task status notifications
type TaskNotificationTx = OneshotSender<TaskStatusNotification>;
/// A oneshot channel on which to receive task status notifications
type TaskNotificationRx = OneshotReceiver<TaskStatusNotification>;

/// A map of task IDs to their corresponding notification channels
type NotificationMap = Arc<RwLock<HashMap<TaskIdentifier, TaskNotificationTx>>>;

/// The future type for a task waiter
type TaskWaiterFuture = BoxFuture<'static, Result<(), RenegadeClientError>>;

// -------------------
// | Channel Helpers |
// -------------------

/// Create a new notification channel
pub fn create_notification_channel() -> (TaskNotificationTx, TaskNotificationRx) {
    oneshot::channel()
}

// ---------
// | Types |
// ---------

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

// -----------------------
// | Task Waiter Manager |
// -----------------------

/// Manages sending notifications to task waiters
#[derive(Clone)]
pub struct TaskWaiterManager {
    /// The notification map
    notifications: NotificationMap,
}

impl TaskWaiterManager {
    /// Create a new task waiter manager
    pub fn new<S>(tasks_topic: S) -> Self
    where
        S: Stream<Item = TaskUpdateWebsocketMessage> + Unpin + Send + 'static,
    {
        let this = Self { notifications: Arc::new(RwLock::new(HashMap::new())) };

        let this_clone = this.clone();
        tokio::spawn(async move { this_clone.watch_task_updates(tasks_topic).await });

        this
    }

    /// Create a task waiter which can be awaited until the given task completes
    pub async fn create_task_waiter(
        &self,
        task_id: TaskIdentifier,
        timeout: Duration,
    ) -> TaskWaiter {
        let (tx, rx) = create_notification_channel();
        self.notifications.write().await.insert(task_id, tx);
        TaskWaiter::new(task_id, rx, timeout)
    }

    /// A persistent loop which watches for task updates and forward the task
    /// status notification to the appropriate receiver if the task's status
    /// is being awaited
    async fn watch_task_updates<S>(&self, mut tasks_topic: S)
    where
        S: Stream<Item = TaskUpdateWebsocketMessage> + Unpin,
    {
        while let Some(message) = tasks_topic.next().await {
            self.handle_task_update(message.task).await;
        }

        error!("Task update stream closed");
    }

    /// Handle a task update, forwarding the task status notification to the
    /// appropriate receiver if the task's status is being awaited
    async fn handle_task_update(&self, task: ApiTask) {
        let ApiTask { id, state, .. } = task;
        let state = state.to_lowercase();
        if state.contains("completed") {
            self.handle_completed_task(id).await;
        } else if state.contains("failed") {
            self.handle_failed_task(id, state).await;
        }
    }

    /// Handle a completed task, forwarding a success notification
    async fn handle_completed_task(&self, task_id: TaskIdentifier) {
        let mut notifications = self.notifications.write().await;

        let tx = match notifications.remove(&task_id) {
            Some(tx) => tx,
            None => return,
        };

        // We explicitly ignore errors here in case the receiver is dropped
        let _ = tx.send(TaskStatusNotification::Success);
    }

    /// Handle a failed task, forwarding a failure notification
    async fn handle_failed_task(&self, task_id: TaskIdentifier, error: String) {
        let mut notifications = self.notifications.write().await;

        let tx = match notifications.remove(&task_id) {
            Some(tx) => tx,
            None => return,
        };

        // We explicitly ignore errors here in case the receiver is dropped
        let _ = tx.send(TaskStatusNotification::Failed { error });
    }
}

// ---------------
// | Task Waiter |
// ---------------

/// A thin wrapper around a notification channel that waits for a task to
/// complete then transforms the status into a result
pub struct TaskWaiter {
    /// The task ID
    task_id: TaskIdentifier,
    /// The task status notification receiver.
    /// This will be `taken` once the task waiter is first polled.
    notification_rx: Option<TaskNotificationRx>,
    /// The duration to wait for the task to complete before timing out
    timeout: Duration,
    /// The underlying future that waits for the task to complete
    fut: Option<TaskWaiterFuture>,
}

impl TaskWaiter {
    /// Create a new task waiter
    pub fn new(
        task_id: TaskIdentifier,
        notification_rx: TaskNotificationRx,
        timeout: Duration,
    ) -> Self {
        Self { task_id, notification_rx: Some(notification_rx), timeout, fut: None }
    }

    /// Watch a task until it terminates as a success or failure
    async fn watch_task(
        task_id: TaskIdentifier,
        notification_rx: TaskNotificationRx,
        timeout: Duration,
    ) -> Result<(), RenegadeClientError> {
        // Register a notification channel for the task and await
        let timeout = tokio::time::timeout(timeout, notification_rx);
        let notification = timeout
            .await
            .map_err(|_| RenegadeClientError::task(task_id, "Task timed out"))?
            .map_err(|_| RenegadeClientError::task(task_id, "Task waiter closed"))?;

        notification.into_result(task_id)
    }
}

impl Future for TaskWaiter {
    type Output = Result<(), RenegadeClientError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.fut.is_none() {
            let notification_rx = this.notification_rx.take().unwrap();
            let fut = Self::watch_task(this.task_id, notification_rx, this.timeout).boxed();
            this.fut = Some(fut);
        }

        this.fut.as_mut().unwrap().as_mut().poll(cx)
    }
}
