//! The websocket client for listening to Renegade events

use std::sync::{Arc, OnceLock};
use std::{collections::HashMap, time::Duration};

use futures_util::stream::SplitSink;
use futures_util::{Sink, SinkExt};
use renegade_api::{
    bus_message::{SystemBusMessage, SystemBusMessageWithTopic as ServerMessage},
    websocket::{ClientWebsocketMessage, WebsocketMessage},
};
use reqwest::header::HeaderMap;
use tokio::net::TcpStream;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    RwLock,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{error, warn};
use uuid::Uuid;

use crate::renegade_api_types::tasks::{ApiTask, TaskIdentifier};
use crate::websocket::subscriptions::{
    SubscriptionManager, SubscriptionRx, SubscriptionTx, TopicRx,
};
use crate::{add_expiring_auth_to_headers, HmacKey};
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

/// A read/write websocket stream
pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
/// A websocket sink (write end)
pub type WsSink = SplitSink<WsStream, Message>;
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
pub fn create_subscription_channel() -> (SubscriptionTx, SubscriptionRx) {
    mpsc::unbounded_channel()
}

/// Construct a websocket topic from a wallet's task history
fn construct_task_history_topic(wallet_id: Uuid) -> String {
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
    /// The topic subscription manager. This is lazily initialized along with
    /// the underlying websocket connection when the first subscription
    /// request is made.
    subscriptions: OnceLock<Arc<SubscriptionManager>>,
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
            subscriptions: OnceLock::new(),
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
        self.ensure_connected();

        // Add a notification channel to the map and create a task waiter
        let (tx, rx) = create_notification_channel();
        self.notifications.write().await.insert(task_id, tx);
        Ok(rx)
    }

    /// Subscribe to a new websocket topic
    async fn subscribe_to_topic(&self, topic: String) -> Result<TopicRx, RenegadeClientError> {
        self.ensure_connected();

        let subscriptions = self.subscriptions.get().unwrap();
        subscriptions.subscribe_to_topic(topic).await
    }

    /// Connect to the websocket server & initialize the subscription manager.
    ///
    /// This will ensure that the websocket connection is only ever initialized
    /// once if it is needed. The initialization spawns a thread which handles
    /// websocket reconnection & subscription management.
    fn ensure_connected(&self) {
        self.subscriptions.get_or_init(|| {
            let (subscriptions_tx, subscriptions_rx) = create_subscription_channel();
            let subscriptions =
                Arc::new(SubscriptionManager::new(self.auth_hmac_key, subscriptions_tx));

            tokio::spawn(Self::ws_reconnection_loop(
                self.base_url.clone(),
                subscriptions.clone(),
                subscriptions_rx,
            ));

            subscriptions
        });
    }

    // ----------------------
    // | Connection Handler |
    // ----------------------

    /// Websocket reconnection loop. Re-establishes the websocket connection
    /// if it could not be established or was closed for any reason.
    async fn ws_reconnection_loop(
        base_url: String,
        subscriptions: Arc<SubscriptionManager>,
        mut subscriptions_rx: SubscriptionRx,
    ) {
        loop {
            let maybe_ws_stream = connect_async(&base_url).await;
            match maybe_ws_stream {
                Ok((ws_stream, _)) => {
                    subscriptions.manage_subscriptions(ws_stream, &mut subscriptions_rx).await;
                },
                Err(e) => {
                    error!("Error connecting to websocket: {e}");
                },
            }

            warn!("Re-establishing websocket connection...");
            tokio::time::sleep(WS_RECONNECTION_DELAY).await;
        }
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
            SystemBusMessage::TaskHistoryUpdate { task: ApiTask { id, state, .. } } => {
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
