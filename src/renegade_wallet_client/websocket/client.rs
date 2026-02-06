//! The websocket client for listening to Renegade events

use std::sync::{Arc, OnceLock};
use std::{collections::HashMap, time::Duration};

use futures_util::Stream;
use futures_util::stream::SplitSink;
use renegade_external_api::types::websocket::ServerWebsocketMessageBody;
use renegade_types_core::HmacKey;
use tokio::net::TcpStream;
use tokio::sync::{
    OnceCell as AsyncOnceCell, RwLock,
    mpsc::{self, UnboundedReceiver, UnboundedSender},
};
use tokio_stream::StreamExt;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, warn};
use uuid::Uuid;

use crate::websocket::subscriptions::{
    SubscriptionManager, SubscriptionRx, SubscriptionTx, TopicStream,
};
use crate::websocket::{TaskWaiter, TaskWaiterManager};
use crate::{RenegadeClientError, renegade_wallet_client::config::RenegadeClientConfig};

// -------------
// | Constants |
// -------------

/// The default port on which relayers run websocket servers
const DEFAULT_WS_PORT: u16 = 4000;

/// The delay between websocket reconnection attempts
const WS_RECONNECTION_DELAY: Duration = Duration::from_secs(1);

/// The admin balances websocket topic
pub const ADMIN_BALANCES_TOPIC: &str = "/v2/admin/balances";

/// The admin orders websocket topic
pub const ADMIN_ORDERS_TOPIC: &str = "/v2/admin/orders";

// ---------
// | Types |
// ---------

/// A read/write websocket stream
pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
/// A websocket sink (write end)
pub type WsSink = SplitSink<WsStream, Message>;

/// A channel on which to request websocket subscriptions
pub type SubscribeTx = UnboundedSender<Uuid>;
/// A channel on which to receive websocket subscriptions
pub type SubscribeRx = UnboundedReceiver<Uuid>;
/// A shared map type
pub type SharedMap<K, V> = Arc<RwLock<HashMap<K, V>>>;

// -----------
// | Helpers |
// -----------

/// Create a new subscription channel
pub fn create_subscription_channel() -> (SubscriptionTx, SubscriptionRx) {
    mpsc::unbounded_channel()
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
    /// The admin HMAC key used to authenticate admin websocket topic
    /// subscriptions
    admin_hmac_key: Option<HmacKey>,
    /// The topic subscription manager. This is lazily initialized along with
    /// the underlying websocket connection when the first subscription
    /// request is made.
    subscriptions: OnceLock<Arc<SubscriptionManager>>,
    /// The task waiter manager. This is lazily initialized when the first task
    /// waiter is created.
    task_waiter_manager: AsyncOnceCell<Arc<TaskWaiterManager>>,
}

impl RenegadeWebsocketClient {
    /// Create a new websocket client
    pub fn new(
        config: &RenegadeClientConfig,
        account_id: Uuid,
        auth_hmac_key: HmacKey,
        admin_hmac_key: Option<HmacKey>,
    ) -> Self {
        let base_url = config.relayer_base_url.replace("http", "ws");
        let base_url = format!("{base_url}:{DEFAULT_WS_PORT}");

        Self {
            base_url,
            account_id,
            auth_hmac_key,
            admin_hmac_key,
            subscriptions: OnceLock::new(),
            task_waiter_manager: AsyncOnceCell::new(),
        }
    }

    // -----------------
    // | Subscriptions |
    // -----------------

    /// Subscribe to a new websocket topic
    async fn subscribe_to_topic(&self, topic: String) -> Result<TopicStream, RenegadeClientError> {
        self.ensure_subscriptions_initialized();

        let subscriptions = self.subscriptions.get().unwrap();
        subscriptions.subscribe_to_topic(topic).await
    }

    // --- Tasks --- //

    /// Subscribe to the account's task updates stream
    pub async fn subscribe_task_updates(
        &self,
    ) -> Result<impl Stream<Item = ServerWebsocketMessageBody> + use<>, RenegadeClientError> {
        let stream = self.subscribe_to_topic(self.tasks_topic()).await?;

        let filtered_stream = stream.filter_map(|maybe_ws_msg| {
            maybe_ws_msg.ok().and_then(|ws_msg| match &ws_msg {
                ServerWebsocketMessageBody::TaskUpdate { .. } => Some(ws_msg),
                _ => None,
            })
        });

        Ok(filtered_stream)
    }

    /// Construct the account's task updates topic name
    fn tasks_topic(&self) -> String {
        format!("/v2/account/{}/tasks", self.account_id)
    }

    // --- Balances --- //

    /// Subscribe to the account's balance updates stream
    pub async fn subscribe_balance_updates(
        &self,
    ) -> Result<impl Stream<Item = ServerWebsocketMessageBody>, RenegadeClientError> {
        let stream = self.subscribe_to_topic(self.balances_topic()).await?;

        let filtered_stream = stream.filter_map(|maybe_ws_msg| {
            maybe_ws_msg.ok().and_then(|ws_msg| match &ws_msg {
                ServerWebsocketMessageBody::BalanceUpdate { .. } => Some(ws_msg),
                _ => None,
            })
        });

        Ok(filtered_stream)
    }

    /// Construct the account's balance updates topic name
    fn balances_topic(&self) -> String {
        format!("/v2/account/{}/balances", self.account_id)
    }

    // --- Orders --- //

    /// Subscribe to the account's order updates stream
    pub async fn subscribe_order_updates(
        &self,
    ) -> Result<impl Stream<Item = ServerWebsocketMessageBody>, RenegadeClientError> {
        let stream = self.subscribe_to_topic(self.orders_topic()).await?;

        let filtered_stream = stream.filter_map(|maybe_ws_msg| {
            maybe_ws_msg.ok().and_then(|ws_msg| match &ws_msg {
                ServerWebsocketMessageBody::OrderUpdate { .. } => Some(ws_msg),
                _ => None,
            })
        });

        Ok(filtered_stream)
    }

    /// Construct the account's order updates topic name
    fn orders_topic(&self) -> String {
        format!("/v2/account/{}/orders", self.account_id)
    }

    // --- Fills --- //

    /// Subscribe to the account's fills stream
    pub async fn subscribe_fills(
        &self,
    ) -> Result<impl Stream<Item = ServerWebsocketMessageBody>, RenegadeClientError> {
        let stream = self.subscribe_to_topic(self.fills_topic()).await?;

        let filtered_stream = stream.filter_map(|maybe_ws_msg| {
            maybe_ws_msg.ok().and_then(|ws_msg| match &ws_msg {
                ServerWebsocketMessageBody::Fill { .. } => Some(ws_msg),
                _ => None,
            })
        });

        Ok(filtered_stream)
    }

    /// Construct the account's fills topic name
    fn fills_topic(&self) -> String {
        format!("/v2/account/{}/fills", self.account_id)
    }

    // --- Admin --- //

    /// Subscribe to the admin balances updates stream
    pub async fn subscribe_admin_balance_updates(
        &self,
    ) -> Result<impl Stream<Item = ServerWebsocketMessageBody>, RenegadeClientError> {
        let stream = self.subscribe_to_topic(ADMIN_BALANCES_TOPIC.to_string()).await?;

        let filtered_stream = stream.filter_map(|maybe_ws_msg| {
            maybe_ws_msg.ok().and_then(|ws_msg| match &ws_msg {
                ServerWebsocketMessageBody::AdminBalanceUpdate { .. } => Some(ws_msg),
                _ => None,
            })
        });

        Ok(filtered_stream)
    }

    /// Subscribe to the admin order updates stream
    pub async fn subscribe_admin_order_updates(
        &self,
    ) -> Result<impl Stream<Item = ServerWebsocketMessageBody>, RenegadeClientError> {
        let stream = self.subscribe_to_topic(ADMIN_ORDERS_TOPIC.to_string()).await?;

        let filtered_stream = stream.filter_map(|maybe_ws_msg| {
            maybe_ws_msg.ok().and_then(|ws_msg| match &ws_msg {
                ServerWebsocketMessageBody::AdminOrderUpdate { .. } => Some(ws_msg),
                _ => None,
            })
        });

        Ok(filtered_stream)
    }

    // ----------------
    // | Task Waiters |
    // ----------------

    /// Subscribe to a new task's status
    pub async fn watch_task(
        &self,
        task_id: Uuid,
        timeout: Duration,
    ) -> Result<TaskWaiter, RenegadeClientError> {
        let task_waiter_manager = self.ensure_task_waiters_initialized().await?;

        Ok(task_waiter_manager.create_task_waiter(task_id, timeout).await)
    }

    /// Ensure that the task waiter manager is initialized with a subscription
    /// to the tasks topic.
    async fn ensure_task_waiters_initialized(
        &self,
    ) -> Result<&Arc<TaskWaiterManager>, RenegadeClientError> {
        let this = self.clone();
        self.task_waiter_manager
            .get_or_try_init(|| async move {
                let tasks_topic = this.subscribe_task_updates().await?;
                Ok(Arc::new(TaskWaiterManager::new(tasks_topic)))
            })
            .await
    }

    // ----------------------
    // | Connection Handler |
    // ----------------------

    /// Connect to the websocket server & initialize the subscription manager.
    ///
    /// This will ensure that the websocket connection is only ever initialized
    /// once if it is needed. The initialization spawns a thread which handles
    /// websocket reconnection & subscription management.
    fn ensure_subscriptions_initialized(&self) {
        self.subscriptions.get_or_init(|| {
            let (subscriptions_tx, subscriptions_rx) = create_subscription_channel();
            let subscriptions = Arc::new(SubscriptionManager::new(
                self.auth_hmac_key,
                self.admin_hmac_key,
                subscriptions_tx,
            ));

            tokio::spawn(Self::ws_reconnection_loop(
                self.base_url.clone(),
                subscriptions.clone(),
                subscriptions_rx,
            ));

            subscriptions
        });
    }

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
}
