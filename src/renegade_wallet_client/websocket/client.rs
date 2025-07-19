//! The websocket client for listening to Renegade events

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::{Sink, SinkExt, StreamExt};
use renegade_api::{
    bus_message::{SystemBusMessage, SystemBusMessageWithTopic as ServerMessage},
    websocket::WebsocketMessage,
};
use renegade_common::types::tasks::TaskIdentifier;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    OnceCell, RwLock,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::error;

use crate::{
    renegade_wallet_client::config::RenegadeClientConfig,
    websocket::task_waiter::{TaskStatusNotification, TaskWaiter},
    RenegadeClientError,
};

// ---------
// | Types |
// ---------

/// A notification channel
///
/// TODO: Add a type for the notification
pub type TaskNotificationTx = UnboundedSender<TaskStatusNotification>;
/// A channel on which to receive task notifications
pub type TaskNotificationRx = UnboundedReceiver<TaskStatusNotification>;
/// A channel on which to request websocket subscriptions
pub type SubscribeTx = UnboundedSender<TaskIdentifier>;
/// A channel on which to receive websocket subscriptions
pub type SubscribeRx = UnboundedReceiver<TaskIdentifier>;
/// A shared map type
pub type SharedMap<K, V> = Arc<RwLock<HashMap<K, V>>>;
/// A notification map
pub type NotificationMap = SharedMap<TaskIdentifier, TaskNotificationTx>;

// -----------
// | Helpers |
// -----------

/// Create a new notification map
pub fn create_notification_map() -> NotificationMap {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Create a new notification channel
pub fn create_notification_channel() -> (TaskNotificationTx, TaskNotificationRx) {
    mpsc::unbounded_channel()
}

/// Create a new subscription channel
pub fn create_subscription_channel() -> (SubscribeTx, SubscribeRx) {
    mpsc::unbounded_channel()
}

/// Construct a websocket topic from a task identifier
fn construct_websocket_topic(task_id: TaskIdentifier) -> String {
    format!("/v0/tasks/{task_id}")
}

// --------------------
// | Websocket Client |
// --------------------

/// The websocket client for listening to Renegade events
#[derive(Debug, Clone)]
pub struct RenegadeWebsocketClient {
    /// The base url of the websocket server
    base_url: String,
    /// The notifications map
    notifications: NotificationMap,
    /// The channel to subscribe to task status updates
    ///
    /// This is used
    subscribe_tx: Arc<OnceCell<SubscribeTx>>,
}

impl RenegadeWebsocketClient {
    /// Create a new websocket client
    pub fn new(config: &RenegadeClientConfig) -> Self {
        let base_url = config.relayer_base_url.replace("http", "ws");
        Self {
            base_url,
            notifications: create_notification_map(),
            subscribe_tx: Arc::new(OnceCell::new()),
        }
    }

    // -----------------
    // | Subscriptions |
    // -----------------

    /// Subscribe to a new task's status
    pub async fn watch_task(
        &self,
        task_id: TaskIdentifier,
    ) -> Result<TaskWaiter, RenegadeClientError> {
        self.ensure_connected().await?;
        // Send a subscription message to the websocket client
        let subscribe_tx = self
            .subscribe_tx
            .get()
            .ok_or(RenegadeClientError::custom("Websocket client not connected"))?;
        subscribe_tx.send(task_id).map_err(RenegadeClientError::custom)?;

        // Add a notification channel to the map and create a task waiter
        let (tx, rx) = create_notification_channel();
        self.notifications.write().await.insert(task_id, tx);
        Ok(TaskWaiter::new(rx))
    }

    /// Connect to the websocket server
    ///
    /// This will ensure that the websocket connection is only ever initialized
    /// once if it is needed. The initialization spawns a thread to watch
    /// subscribed topics and forward them to threads watching for
    /// notifications.
    pub async fn ensure_connected(&self) -> Result<(), RenegadeClientError> {
        self.subscribe_tx
            .get_or_try_init(|| async {
                let (tx, rx) = create_subscription_channel();
                let base_url = self.base_url.clone();
                let notifications = self.notifications.clone();
                tokio::spawn(async move {
                    if let Err(e) = Self::handle_ws_connection(base_url, rx, notifications).await {
                        error!("Failed to handle websocket connection: {e}");
                    }
                });

                Ok(tx)
            })
            .await?;
        Ok(())
    }

    // ----------------------
    // | Connection Handler |
    // ----------------------

    /// Connection handler loop
    async fn handle_ws_connection(
        base_url: String,
        mut subscribe_rx: SubscribeRx,
        notifications: NotificationMap,
    ) -> Result<(), RenegadeClientError> {
        let (ws_stream, _response) = connect_async(&base_url).await.map_err(|e| {
            RenegadeClientError::custom(format!("Failed to connect to websocket: {e}"))
        })?;

        // Split the websocket stream into sink and stream
        let (mut ws_tx, mut ws_rx) = ws_stream.split();

        loop {
            tokio::select! {
                Some(task_id) = subscribe_rx.recv() => {
                    if let Err(e) = Self::subscribe_to_task(task_id, &mut ws_tx).await {
                        error!("Failed to subscribe to task {task_id}: {e}");
                    }
                }
                Some(msg_res) = ws_rx.next() => {
                    let msg = match msg_res {
                        Ok(msg) => msg,
                        Err(e) => {
                            error!("Failed to receive websocket message: {e}");
                            continue;
                        }
                    };

                    // Handle the incoming message
                    if let Err(e) = Self::handle_incoming_message(msg, notifications.clone()).await {
                        error!("Failed to handle incoming websocket message: {e}");
                    }
                }
                else => break,
            }
        }

        Ok(())
    }

    /// Subscribe to a new task's status
    async fn subscribe_to_task<W: Sink<Message> + Unpin>(
        task_id: TaskIdentifier,
        ws_tx: &mut W,
    ) -> Result<(), RenegadeClientError> {
        let topic = construct_websocket_topic(task_id);
        let subscribe = WebsocketMessage::Subscribe { topic };
        let msg_txt = serde_json::to_string(&subscribe).map_err(RenegadeClientError::serde)?;
        let msg = Message::Text(msg_txt);

        // Send the message onto the websocket
        ws_tx
            .send(msg)
            .await
            .map_err(|_| RenegadeClientError::websocket("failed to send websocket message"))
    }

    /// Handle an incoming websocket message
    async fn handle_incoming_message(
        msg: Message,
        notifications: NotificationMap,
    ) -> Result<(), RenegadeClientError> {
        let msg: ServerMessage = match msg {
            Message::Text(txt) => serde_json::from_str(&txt).map_err(RenegadeClientError::serde)?,
            _ => return Ok(()),
        };

        // Handle the message
        match msg.event {
            SystemBusMessage::TaskStatusUpdate { status } => {
                let id = status.id;
                let state = status.state;
                let committed = status.committed;
                if committed {
                    Self::handle_completed_task(id, notifications).await?;
                } else if state.to_lowercase().contains("failed") {
                    Self::handle_failed_task(id, state, notifications).await?;
                }
            },
            _ => return Ok(()),
        }
        todo!()
    }

    /// Handle a completed task
    async fn handle_completed_task(
        task_id: TaskIdentifier,
        notifications: NotificationMap,
    ) -> Result<(), RenegadeClientError> {
        let mut notif_map = notifications.write().await;
        let tx = match notif_map.remove(&task_id) {
            Some(tx) => tx.clone(),
            None => return Ok(()),
        };

        // We explicitly ignore errors here in case the receiver is dropped
        let _ = tx.send(TaskStatusNotification::Success);
        Ok(())
    }

    /// Handle a failed task
    async fn handle_failed_task(
        task_id: TaskIdentifier,
        error: String,
        notifications: NotificationMap,
    ) -> Result<(), RenegadeClientError> {
        let mut notif_map = notifications.write().await;
        let tx = match notif_map.remove(&task_id) {
            Some(tx) => tx.clone(),
            None => return Ok(()),
        };

        // We explicitly ignore errors here in case the receiver is dropped
        let _ = tx.send(TaskStatusNotification::Failed { error });
        Ok(())
    }
}
