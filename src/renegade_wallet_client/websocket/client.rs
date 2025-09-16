//! The websocket client for listening to Renegade events

use std::sync::Arc;
use std::{collections::HashMap, time::Duration};

use futures_util::{Sink, SinkExt, StreamExt};
use renegade_api::{
    bus_message::{SystemBusMessage, SystemBusMessageWithTopic as ServerMessage},
    websocket::{ClientWebsocketMessage, WebsocketMessage},
};
use renegade_common::types::tasks::TaskIdentifier;
use renegade_common::types::wallet::WalletIdentifier;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    OnceCell, RwLock,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, warn};

use crate::actions::get_task_history::get_task_history;
use crate::http::RelayerHttpClient;
use crate::{
    renegade_wallet_client::config::RenegadeClientConfig,
    websocket::task_waiter::TaskStatusNotification, RenegadeClientError,
};

// ---------
// | Types |
// ---------

/// The default port on which relayers run websocket servers
const DEFAULT_WS_PORT: u16 = 4000;

/// The delay between websocket reconnection attempts
const WS_RECONNECTION_DELAY: Duration = Duration::from_secs(1);

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
#[derive(Clone)]
pub struct RenegadeWebsocketClient {
    /// The base url of the websocket server
    base_url: String,
    /// The wallet ID
    wallet_id: WalletIdentifier,
    /// The historical state client, used to check task history in the case of
    /// missed updates
    historical_state_client: Arc<RelayerHttpClient>,
    /// The notifications map
    notifications: NotificationMap,
    /// The channel to subscribe to task status updates
    ///
    /// This is used to send subscription messages to the websocket server.
    subscribe_tx: Arc<OnceCell<SubscribeTx>>,
}

impl RenegadeWebsocketClient {
    /// Create a new websocket client
    pub fn new(
        config: &RenegadeClientConfig,
        wallet_id: WalletIdentifier,
        historical_state_client: Arc<RelayerHttpClient>,
    ) -> Self {
        let base_url = config.relayer_base_url.replace("http", "ws");
        let base_url = format!("{base_url}:{DEFAULT_WS_PORT}");

        Self {
            base_url,
            wallet_id,
            historical_state_client,
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
    ) -> Result<TaskNotificationRx, RenegadeClientError> {
        self.ensure_connected().await;
        // Send a subscription message to the websocket client
        let subscribe_tx = self
            .subscribe_tx
            .get()
            .ok_or(RenegadeClientError::custom("Websocket client not connected"))?;
        subscribe_tx.send(task_id).map_err(RenegadeClientError::custom)?;

        // Add a notification channel to the map and create a task waiter
        let (tx, rx) = create_notification_channel();
        self.notifications.write().await.insert(task_id, tx);
        Ok(rx)
    }

    /// Connect to the websocket server
    ///
    /// This will ensure that the websocket connection is only ever initialized
    /// once if it is needed. The initialization spawns a thread to watch
    /// subscribed topics and forward them to threads watching for
    /// notifications.
    pub async fn ensure_connected(&self) {
        self.subscribe_tx
            .get_or_init(|| async {
                let (tx, rx) = create_subscription_channel();
                let self_clone = self.clone();
                tokio::spawn(self_clone.ws_reconnection_loop(rx));

                tx
            })
            .await;
    }

    // ----------------------
    // | Connection Handler |
    // ----------------------

    /// Websocket reconnection loop. Re-establishes the websocket connection
    /// if there is an error in handling it.
    async fn ws_reconnection_loop(self, mut subscribe_rx: SubscribeRx) {
        loop {
            if let Err(e) = self.handle_ws_connection(&mut subscribe_rx).await {
                error!("Error handling websocket connection: {e}");
            }

            warn!("Re-establishing websocket connection & re-subscribing to task updates...");
            tokio::time::sleep(WS_RECONNECTION_DELAY).await;
        }
    }

    /// Connection handler loop
    async fn handle_ws_connection(
        &self,
        subscribe_rx: &mut SubscribeRx,
    ) -> Result<(), RenegadeClientError> {
        let (ws_stream, _response) = connect_async(&self.base_url).await.map_err(|e| {
            RenegadeClientError::custom(format!("Failed to connect to websocket: {e}"))
        })?;

        // Split the websocket stream into sink and stream
        let (mut ws_tx, mut ws_rx) = ws_stream.split();

        // Re-subscribe to all tasks in the notification map
        self.resubscribe_to_all_tasks(&mut ws_tx).await?;

        loop {
            tokio::select! {
                Some(task_id) = subscribe_rx.recv() => Self::subscribe_to_task(task_id, &mut ws_tx).await?,
                Some(Ok(msg)) = ws_rx.next() => {
                    let txt = match msg {
                        Message::Text(txt) => txt,
                        Message::Close(_) => {
                            warn!("Websocket connection closed");
                            break;
                        },
                        _ => continue,
                    };

                    // Handle the incoming message
                    if let Err(e) = self.handle_incoming_message(txt).await {
                        error!("Failed to handle incoming websocket message: {e}");
                    }
                }
                else => break,
            }
        }

        Ok(())
    }

    /// Resubscribe to all tasks in the notification map
    async fn resubscribe_to_all_tasks<W: Sink<Message> + Unpin>(
        &self,
        ws_tx: &mut W,
    ) -> Result<(), RenegadeClientError> {
        let notif_map = self.notifications.read().await;
        let task_ids: Vec<TaskIdentifier> = notif_map.keys().cloned().collect();
        drop(notif_map);

        for task_id in task_ids {
            Self::subscribe_to_task(task_id, ws_tx).await?;
        }

        Ok(())
    }

    /// Subscribe to a new task's status
    async fn subscribe_to_task<W: Sink<Message> + Unpin>(
        task_id: TaskIdentifier,
        ws_tx: &mut W,
    ) -> Result<(), RenegadeClientError> {
        let topic = construct_websocket_topic(task_id);
        let headers = HashMap::new();
        let subscribe =
            ClientWebsocketMessage { headers, body: WebsocketMessage::Subscribe { topic } };
        let msg_txt = serde_json::to_string(&subscribe).map_err(RenegadeClientError::serde)?;

        // Send the message onto the websocket
        ws_tx
            .send(Message::Text(msg_txt))
            .await
            .map_err(|_| RenegadeClientError::websocket("failed to send websocket message"))
    }

    /// Handle an incoming websocket message
    async fn handle_incoming_message(&self, txt: String) -> Result<(), RenegadeClientError> {
        let msg: ServerMessage =
            match serde_json::from_str(&txt).map_err(RenegadeClientError::serde) {
                Ok(msg) => msg,
                Err(e) => {
                    // If the message contains the "subscriptions" substring, we interpret this as a
                    // subscriptions response, and return early
                    if txt.contains("subscriptions") {
                        return Ok(());
                    }

                    // If the message contains the "task not found" substring, we interpret this as
                    // a task having completed, so we check task history to track the
                    // success/failure of all completed tasks
                    if txt.contains("task not found") {
                        warn!("Task not found in relayer, checking task history...");
                        return self.handle_historic_tasks().await;
                    }

                    return Err(e);
                },
            };

        // Handle the message
        match msg.event {
            SystemBusMessage::TaskStatusUpdate { status } => {
                let id = status.id;
                let state = status.state.to_lowercase();
                if state.contains("completed") {
                    self.handle_completed_task(id).await?;
                } else if state.contains("failed") {
                    self.handle_failed_task(id, state).await?;
                }
            },
            _ => return Ok(()),
        }
        Ok(())
    }

    /// Handle a completed task
    async fn handle_completed_task(
        &self,
        task_id: TaskIdentifier,
    ) -> Result<(), RenegadeClientError> {
        let mut notif_map = self.notifications.write().await;
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
        &self,
        task_id: TaskIdentifier,
        error: String,
    ) -> Result<(), RenegadeClientError> {
        let mut notif_map = self.notifications.write().await;
        let tx = match notif_map.remove(&task_id) {
            Some(tx) => tx.clone(),
            None => return Ok(()),
        };

        // We explicitly ignore errors here in case the receiver is dropped
        let _ = tx.send(TaskStatusNotification::Failed { error });
        Ok(())
    }

    /// Handle historic tasks that may have been missed by the websocket client
    async fn handle_historic_tasks(&self) -> Result<(), RenegadeClientError> {
        let task_history = get_task_history(&self.historical_state_client, self.wallet_id).await?;

        let notif_map = self.notifications.read().await;
        let task_ids: Vec<TaskIdentifier> = notif_map.keys().cloned().collect();
        drop(notif_map);

        for task_id in task_ids {
            let task = task_history.iter().find(|task| task.id == task_id);
            if let Some(task) = task {
                let state = task.state.to_lowercase();
                if state.contains("completed") {
                    self.handle_completed_task(task_id).await?;
                } else if state.contains("failed") {
                    self.handle_failed_task(task_id, state).await?;
                }
            }
        }

        Ok(())
    }
}
