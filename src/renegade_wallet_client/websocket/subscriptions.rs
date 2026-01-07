//! The websocket client's subscriptions manager, which handles subscribing to
//! different relayer topics and streaming them out separately

use std::{collections::HashMap, time::Duration};

use futures_util::{SinkExt, StreamExt};
use reqwest::header::HeaderMap;
use tokio::sync::{
    broadcast::{self, Receiver as BroadcastReceiver, Sender as BroadcastSender},
    mpsc::{UnboundedReceiver, UnboundedSender},
    RwLock,
};
use tokio_stream::wrappers::BroadcastStream;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};

use crate::{
    add_expiring_auth_to_headers,
    renegade_api_types::websocket::{
        ClientWebsocketMessage, ClientWebsocketMessageBody, ServerWebsocketMessage,
        ServerWebsocketMessageBody,
    },
    websocket::{WsSink, WsStream, ADMIN_BALANCES_TOPIC, ADMIN_ORDERS_TOPIC},
    HmacKey, RenegadeClientError,
};

// -------------
// | Constants |
// -------------

/// The expiration duration for websocket subscription authentication
const AUTH_EXPIRATION: Duration = Duration::from_secs(10);

// ---------
// | Types |
// ---------

/// A channel on which to forward server websocket messages for a given topic
pub type TopicTx = BroadcastSender<ServerWebsocketMessageBody>;
/// A channel on which to receive forwarded server websocket messages for a
/// given topic
pub type TopicRx = BroadcastReceiver<ServerWebsocketMessageBody>;
/// A stream wrapper around the receiver end of a topic channel
pub type TopicStream = BroadcastStream<ServerWebsocketMessageBody>;

/// A channel on which to send client subscribe/unsubscribe messages
pub type SubscriptionTx = UnboundedSender<ClientWebsocketMessageBody>;
/// A channel on which to receive client subscribe/unsubscribe messages to be
/// forwarded to the server
pub type SubscriptionRx = UnboundedReceiver<ClientWebsocketMessageBody>;

/// A map of topics to their corresponding forwarder channels
pub type SubscriptionMap = RwLock<HashMap<String, TopicTx>>;

// -------------------
// | Channel Helpers |
// -------------------

/// Create a new topic channel. We use a broadcast channel to allow multiple
/// listeners to subscribe to the same topic. We use a buffer size of 100 to
/// allow for some backpressure in case the listeners are not able to process
/// messages fast enough.
fn create_topic_channel() -> (TopicTx, TopicRx) {
    broadcast::channel(100)
}

// ------------------------
// | Subscription Manager |
// ------------------------

/// Manages client subscriptions to websocket API topics
pub struct SubscriptionManager {
    /// The account's HMAC key
    auth_hmac_key: HmacKey,
    /// The admin HMAC key used to authenticate admin websocket topic
    /// subscriptions
    admin_hmac_key: Option<HmacKey>,
    /// The channel on which to enqueue subscription requests to be forwarded to
    /// the server
    subscriptions_tx: SubscriptionTx,
    /// The map of subscribed topics
    subscribed_topics: SubscriptionMap,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    pub fn new(
        auth_hmac_key: HmacKey,
        admin_hmac_key: Option<HmacKey>,
        subscriptions_tx: SubscriptionTx,
    ) -> Self {
        Self {
            auth_hmac_key,
            admin_hmac_key,
            subscriptions_tx,
            subscribed_topics: RwLock::new(HashMap::new()),
        }
    }

    /// Subscribe to the given topic
    pub async fn subscribe_to_topic(
        &self,
        topic: String,
    ) -> Result<TopicStream, RenegadeClientError> {
        // If there is already an active subscription for the topic, return the existing
        // topic channel
        if let Some(tx) = self.try_get_subscription(&topic).await {
            return Ok(tx.subscribe().into());
        }

        // Forward the subscription request to the server
        self.request_subscribe(topic.clone()).await?;

        // Optimistically insert a topic channel into the map for the new subscription.
        // TODO: Have this method await a subscription response from the server before
        // creating the topic channel
        Ok(self.insert_new_subscription(topic).await.into())
    }

    /// Unsubscribe from the given topic
    #[allow(dead_code)]
    pub async fn unsubscribe_from_topic(&self, topic: String) -> Result<(), RenegadeClientError> {
        match self.try_get_subscription(&topic).await {
            // If there are still listeners for the topic, do nothing
            Some(tx) => {
                let receiver_count = tx.receiver_count();
                if receiver_count > 0 {
                    warn!("There are still {receiver_count} listeners for topic {topic}, retaining subscription");
                    return Ok(());
                }
            },
            // If there is no active subscription for the topic, do nothing
            None => return Ok(()),
        }

        // Forward the request to unsubscribe from the topic to the server
        self.request_unsubscribe(topic.clone()).await?;

        // Optimistically remove the subscription from the map
        // TODO: Have this method await a subscription response from the server before
        // removing the subscription from the map
        self.remove_subscription(topic).await;

        Ok(())
    }

    /// Persistent loop that manages client subscriptions to websocket API
    /// topics.
    ///
    /// Listens for subscription requests on the given `SubscriptionRx` &
    /// forwards them to the server via the given `WsStream`.
    /// Then listens for server messages on the `WsStream` & forwards them to
    /// the appropriate topic channels.
    ///
    /// Returns when either the websocket connection or the subscription request
    /// channel is closed.
    pub async fn manage_subscriptions(
        &self,
        ws_stream: WsStream,
        subscriptions_rx: &mut SubscriptionRx,
    ) {
        let (mut ws_tx, mut ws_rx) = ws_stream.split();

        // Re-send subscription requests to the server for all active subscriptions
        self.resubscribe_to_all_topics().await.unwrap();

        loop {
            tokio::select! {
                // Handle incoming subscription requests from the client.
                // We don't handle the case where `subscriptions_rx.recv()` returns `None`,
                // because we store the send handle of this channel directly on `self`,
                // so we know it has not been dropped.
                Some(msg) = subscriptions_rx.recv() => {
                    // Client subscription request received successfully
                    if let Err(e) = self.forward_client_subscription_request(msg, &mut ws_tx).await {
                        error!("Error forwarding client subscription request: {e}");
                        continue;
                    }
                }

                // Handle incoming websocket messages from the server
                maybe_server_msg = ws_rx.next() => {
                    match maybe_server_msg {
                        Some(Ok(msg)) => {
                            // Server message received successfully
                            match msg {
                                Message::Text(txt) => {
                                    // Handle incoming server message
                                    if let Err(e) = self.handle_server_message(txt).await {
                                        error!("Error handling websocket server message: {e}");
                                        continue;
                                    }
                                },
                                Message::Close(frame) => {
                                    warn!("Websocket connection closed");
                                    if let Some(frame) = frame {
                                        warn!("Closure code: {}; closure reason: {}", frame.code, frame.reason);
                                    }

                                    break;
                                },
                                _ => continue,
                            }
                        },
                        Some(Err(e)) => {
                            // Stream still open, error receiving message
                            error!("Error receiving message from websocket: {e}");
                            break;
                        },
                        None => {
                            // Stream closed
                            warn!("Websocket stream closed");
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Re-send subscription requests to the server for all active subscriptions
    async fn resubscribe_to_all_topics(&self) -> Result<(), RenegadeClientError> {
        let subscriptions = self.subscribed_topics.read().await;
        let topics = subscriptions.keys();
        if topics.len() == 0 {
            return Ok(());
        }

        info!("Resubscribing to all topics");
        for topic in topics {
            self.request_subscribe(topic.clone()).await?;
        }

        Ok(())
    }

    /// Send a subscription request for the given topic on the manager's
    /// subscription channel
    async fn request_subscribe(&self, topic: String) -> Result<(), RenegadeClientError> {
        self.subscriptions_tx
            .send(ClientWebsocketMessageBody::Subscribe { topic })
            .map_err(RenegadeClientError::subscription)
    }

    /// Send a request to unsubscribe from the given topic on the manager's
    /// subscription channel
    async fn request_unsubscribe(&self, topic: String) -> Result<(), RenegadeClientError> {
        self.subscriptions_tx
            .send(ClientWebsocketMessageBody::Unsubscribe { topic })
            .map_err(RenegadeClientError::subscription)
    }

    /// Forward a subscription request from the client to the server
    async fn forward_client_subscription_request(
        &self,
        body: ClientWebsocketMessageBody,
        ws_tx: &mut WsSink,
    ) -> Result<(), RenegadeClientError> {
        let body_ser = serde_json::to_vec(&body).map_err(RenegadeClientError::serde)?;
        let mut headers = HeaderMap::new();

        // Determine which HMAC key to use based on whether this is an admin topic
        let hmac_key = if self.is_admin_topic(body.topic()) {
            self.admin_hmac_key.as_ref().ok_or(RenegadeClientError::NotAdmin)?
        } else {
            &self.auth_hmac_key
        };

        add_expiring_auth_to_headers(
            body.topic(),
            &mut headers,
            &body_ser,
            hmac_key,
            AUTH_EXPIRATION,
        );

        let headers = header_map_to_hash_map(headers);

        let msg = ClientWebsocketMessage { headers, body };
        let msg_txt = serde_json::to_string(&msg).map_err(RenegadeClientError::serde)?;

        ws_tx.send(Message::Text(msg_txt)).await.map_err(RenegadeClientError::websocket)?;

        Ok(())
    }

    /// Handle an incoming server message, routing it to the appropriate topic
    /// channel
    async fn handle_server_message(&self, txt: String) -> Result<(), RenegadeClientError> {
        let msg: ServerWebsocketMessage =
            serde_json::from_str(&txt).map_err(RenegadeClientError::serde)?;

        if let Some(tx) = self.try_get_subscription(&msg.topic).await {
            tx.send(msg.body).map_err(RenegadeClientError::subscription)?;
        }

        Ok(())
    }

    /// Get a handle to the topic channel for the given topic, if there is
    /// already an active subscription for it
    async fn try_get_subscription(&self, topic: &str) -> Option<TopicTx> {
        self.subscribed_topics.read().await.get(topic).cloned()
    }

    /// Create a topic channel for the given topic and insert it into the map,
    /// returning the read handle to the channel
    async fn insert_new_subscription(&self, topic: String) -> TopicRx {
        let (tx, rx) = create_topic_channel();
        self.subscribed_topics.write().await.insert(topic, tx);
        rx
    }

    /// Remove a subscription from the map
    async fn remove_subscription(&self, topic: String) {
        self.subscribed_topics.write().await.remove(&topic);
    }

    /// Check if the given topic is an admin topic
    fn is_admin_topic(&self, topic: &str) -> bool {
        topic == ADMIN_BALANCES_TOPIC || topic == ADMIN_ORDERS_TOPIC
    }
}

// -----------
// | Helpers |
// -----------

/// Convert an `http::HeaderMap` to a `HashMap`
fn header_map_to_hash_map(header_map: HeaderMap) -> HashMap<String, String> {
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
