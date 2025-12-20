//! The websocket client for listening to Renegade events

use std::sync::{Arc, OnceLock};
use std::{collections::HashMap, time::Duration};

use futures_util::stream::SplitSink;
use tokio::net::TcpStream;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    OnceCell as AsyncOnceCell, RwLock,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{error, warn};
use uuid::Uuid;

use crate::renegade_api_types::tasks::TaskIdentifier;
use crate::websocket::subscriptions::{
    SubscriptionManager, SubscriptionRx, SubscriptionTx, TopicStream,
};
use crate::websocket::{TaskWaiter, TaskWaiterManager};
use crate::HmacKey;
use crate::{renegade_wallet_client::config::RenegadeClientConfig, RenegadeClientError};

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

/// A channel on which to request websocket subscriptions
pub type SubscribeTx = UnboundedSender<TaskIdentifier>;
/// A channel on which to receive websocket subscriptions
pub type SubscribeRx = UnboundedReceiver<TaskIdentifier>;
/// A shared map type
pub type SharedMap<K, V> = Arc<RwLock<HashMap<K, V>>>;

// -----------
// | Helpers |
// -----------

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
    pub fn new(config: &RenegadeClientConfig, account_id: Uuid, auth_hmac_key: HmacKey) -> Self {
        let base_url = config.relayer_base_url.replace("http", "ws");
        let base_url = format!("{base_url}:{DEFAULT_WS_PORT}");

        Self {
            base_url,
            account_id,
            auth_hmac_key,
            subscriptions: OnceLock::new(),
            task_waiter_manager: AsyncOnceCell::new(),
        }
    }

    // ----------------
    // | Task Waiters |
    // ----------------

    /// Subscribe to a new task's status
    pub async fn watch_task(
        &self,
        task_id: TaskIdentifier,
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
        self.task_waiter_manager
            .get_or_try_init(|| async {
                let tasks_topic = self.subscribe_to_topic(self.tasks_topic()).await?;
                Ok(Arc::new(TaskWaiterManager::new(tasks_topic)))
            })
            .await
    }

    /// Construct the account's task topic name
    fn tasks_topic(&self) -> String {
        format!("/v2/account/{}/tasks", self.account_id)
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
