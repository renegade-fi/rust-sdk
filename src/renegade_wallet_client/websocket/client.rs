//! The websocket client for listening to Renegade events

use std::sync::Arc;
use std::{collections::HashMap, time::Duration};

use futures_util::{Sink, SinkExt, StreamExt};
use renegade_api::auth::add_expiring_auth_to_headers;
use renegade_api::types::ApiHistoricalTask;
use renegade_api::{
    bus_message::{SystemBusMessage, SystemBusMessageWithTopic as ServerMessage},
    websocket::{ClientWebsocketMessage, WebsocketMessage},
};
use renegade_common::types::hmac::HmacKey;
use renegade_common::types::tasks::TaskIdentifier;
use reqwest::header::HeaderMap;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    OnceCell, RwLock,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    renegade_wallet_client::config::RenegadeClientConfig,
    websocket::task_waiter::TaskStatusNotification, RenegadeClientError,
};

// -------------
// | Constants |
// -------------

/// The default port on which relayers run websocket servers
const DEFAULT_WS_PORT: u16 = 4000;

/// The delay between websocket reconnection attempts
const WS_RECONNECTION_DELAY: Duration = Duration::from_secs(1);

/// The expiration duration for websocket subscription authentication
const AUTH_EXPIRATION: Duration = Duration::from_secs(10);

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

/// Construct a websocket topic from a wallet's task history
fn construct_task_history_topic(wallet_id: WalletIdentifier) -> String {
    format!("/v0/wallet/{wallet_id}/task-history")
}

// --------------------
// | Websocket Client |
// --------------------

/// The websocket client for listening to Renegade events
#[derive(Clone)]
pub struct RenegadeWebsocketClient {
    /// The base url of the websocket server
    base_url: String,
    /// The account ID
    account_id: Uuid,
    /// The account's HMAC key
    auth_hmac_key: HmacKey,
    /// The notifications map
    notifications: NotificationMap,
    /// A guard used to ensure that the websocket connection is only ever
    /// initialized once
    connection_guard: Arc<OnceCell<()>>,
}

impl RenegadeWebsocketClient {
    /// Create a new websocket client
    pub fn new(config: &RenegadeClientConfig, account_id: Uuid, auth_hmac_key: HmacKey) -> Self {
        let base_url = config.relayer_base_url.replace("http", "ws");
        let base_url = format!("{base_url}:{DEFAULT_WS_PORT}");

        Self {
            base_url,
            account_id,
            auth_hmac_key,
            notifications: create_notification_map(),
            connection_guard: Arc::new(OnceCell::new()),
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
        self.connection_guard
            .get_or_init(|| async {
                let self_clone = self.clone();
                tokio::spawn(self_clone.ws_reconnection_loop());
            })
            .await;
    }

    // ----------------------
    // | Connection Handler |
    // ----------------------

    /// Websocket reconnection loop. Re-establishes the websocket connection
    /// if there is an error in handling it.
    async fn ws_reconnection_loop(self) {
        loop {
            if let Err(e) = self.handle_ws_connection().await {
                error!("Error handling websocket connection: {e}");
            }

            warn!("Re-establishing websocket connection & re-subscribing to task updates...");
            tokio::time::sleep(WS_RECONNECTION_DELAY).await;
        }
    }

    /// Connection handler loop
    async fn handle_ws_connection(&self) -> Result<(), RenegadeClientError> {
        let (ws_stream, _response) = connect_async(&self.base_url).await.map_err(|e| {
            RenegadeClientError::custom(format!("Failed to connect to websocket: {e}"))
        })?;

        // Split the websocket stream into sink and stream
        let (mut ws_tx, mut ws_rx) = ws_stream.split();

        self.subscribe_to_task_history(&mut ws_tx).await?;

        while let Some(Ok(msg)) = ws_rx.next().await {
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

        Ok(())
    }

    /// Subscribe to the task history of the given wallet
    async fn subscribe_to_task_history<W: Sink<Message> + Unpin>(
        &self,
        ws_tx: &mut W,
    ) -> Result<(), RenegadeClientError> {
        let topic = construct_task_history_topic(self.account_id);

        let body = WebsocketMessage::Subscribe { topic: topic.clone() };

        let body_ser = serde_json::to_vec(&body).map_err(RenegadeClientError::serde)?;
        let mut headers = HeaderMap::new();
        add_expiring_auth_to_headers(
            &topic,
            &mut headers,
            &body_ser,
            &self.auth_hmac_key,
            AUTH_EXPIRATION,
        );

        let headers = header_map_to_hash_map(headers);

        let subscribe = ClientWebsocketMessage { headers, body };

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

                    return Err(e);
                },
            };

        // Handle the message
        match msg.event {
            SystemBusMessage::TaskHistoryUpdate { task: ApiHistoricalTask { id, state, .. } } => {
                let state = state.to_lowercase();
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
}

// -----------
// | Helpers |
// -----------

/// Convert an `http::HeaderMap` to a `HashMap`
pub fn header_map_to_hash_map(header_map: HeaderMap) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    for (k, v) in header_map.into_iter() {
        if let Some(k) = k
            && let Ok(v) = v.to_str()
        {
            headers.insert(k.to_string(), v.to_string());
        }
    }

    headers
}
