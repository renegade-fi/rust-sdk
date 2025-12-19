//! The websocket client's subscriptions manager, which handles subscribing to
//! different relayer topics and streaming them out separately

use std::collections::HashMap;

use futures_util::StreamExt;
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    RwLock,
};
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, warn};

use crate::{
    renegade_api_types::websocket::{ClientWebsocketMessageBody, ServerWebsocketMessageBody},
    websocket::WsStream,
    RenegadeClientError,
};

// ----------------
// | Type Aliases |
// ----------------

/// A channel on which to send client subscribe/unsubscribe messages
pub type SubscriptionTx = UnboundedSender<ClientWebsocketMessageBody>;
/// A channel on which to receive client subscribe/unsubscribe messages, to
/// forward to the server
pub type SubscriptionRx = UnboundedReceiver<ClientWebsocketMessageBody>;
/// A channel on which to forward server websocket messages for a given topic
pub type TopicTx = UnboundedSender<ServerWebsocketMessageBody>;
/// A channel on which to receive forwarded server websocket messages for a
/// given topic
pub type TopicRx = UnboundedReceiver<ServerWebsocketMessageBody>;
/// A map of topics to their corresponding forwarder channels
pub type SubscriptionMap = RwLock<HashMap<String, TopicTx>>;

// -----------
// | Helpers |
// -----------

/// Create a new topic channel
fn create_topic_channel() -> (TopicTx, TopicRx) {
    mpsc::unbounded_channel()
}

// ------------------------
// | Subscription Manager |
// ------------------------

/// Manages client subscriptions to websocket API topics
pub struct SubscriptionManager {
    /// The channel on which to enqueue subscription requests to be forwarded to
    /// the server
    subscriptions_tx: SubscriptionTx,
    /// The map of subscribed topics
    subscribed_topics: SubscriptionMap,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    pub fn new(subscriptions_tx: SubscriptionTx) -> Self {
        Self { subscriptions_tx, subscribed_topics: RwLock::new(HashMap::new()) }
    }

    pub async fn subscribe_to_topic(&self, topic: String) -> Result<TopicRx, RenegadeClientError> {
        // Create the channel on which to stream server messages for the given topic to
        // the client
        let (tx, rx) = create_topic_channel();
        self.subscribed_topics.write().await.insert(topic.clone(), tx);

        // Enqueue the topic subscription request to be forwarded to the server
        self.subscriptions_tx
            .send(ClientWebsocketMessageBody::Subscribe { topic })
            .map_err(RenegadeClientError::subscription)?;

        Ok(rx)
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

        // TODO: Check if there are already subscriptions in the map, which we must
        // re-send subscription requests for

        loop {
            tokio::select! {
                // Handle incoming subscription requests from the client
                maybe_client_msg = subscriptions_rx.recv() => {
                    match maybe_client_msg {
                        Some(msg) => {
                            // Client subscription request received successfully
                            todo!()
                        }
                        None => {
                            // Client subscription request channel closed
                            warn!("Client subscription request channel closed");

                            // TODO: Clean up all subscriptions

                            break;
                        }
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
                                    todo!()
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
}
