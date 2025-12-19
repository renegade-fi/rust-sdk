//! Websocket API types

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::renegade_api_types::{
    balances::ApiBalance,
    orders::{ApiOrder, ApiOrderCore, ApiPartialOrderFill},
    tasks::ApiTask,
};

/// The wrapper websocket message type that contains both a header and body
#[derive(Clone, Debug, Serialize)]
pub struct ClientWebsocketMessage {
    /// The headers associated with the client message
    pub headers: HashMap<String, String>,
    /// The body of the message
    pub body: ClientWebsocketMessageBody,
}

/// A message type that indicates the client would like to either subscribe or
/// unsubscribe from a given topic
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "method", rename_all = "lowercase")]
pub enum ClientWebsocketMessageBody {
    /// Indicates that the client would like to subscribe to the given topic
    Subscribe {
        /// The topic being subscribed to
        topic: String,
    },
    /// Indicates that the client would like to unsubscribe to the given topic
    Unsubscribe {
        /// The topic being unsubscribed from
        topic: String,
    },
}

/// The message type that is sent by the server to the client
#[derive(Clone, Debug, Deserialize)]
pub struct ServerWebsocketMessage {
    /// The topic the message was sent on
    pub topic: String,
    /// The body of the message
    pub body: ServerWebsocketMessageBody,
}

/// The body of a server websocket message
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ServerWebsocketMessageBody {
    /// A message that is sent in response to a subscribe/unsubscribe message,
    /// notifies the client of the now active subscriptions after a
    /// subscribe/unsubscribe message is applied
    Subscriptions {
        /// The current set of topics to which the client is subscribed
        subscriptions: Vec<String>,
    },
    /// A message that is sent when a balance update occurs
    BalanceUpdate {
        /// The updated balance
        balance: ApiBalance,
    },
    /// A message that is sent when an order update occurs
    OrderUpdate {
        /// The updated order
        order: ApiOrder,
    },
    /// A message that is sent when a fill occurs on an order
    Fill {
        /// The fill
        fill: ApiPartialOrderFill,
        /// The order to which the fill pertains
        order: ApiOrderCore,
        /// Whether the order has been entirely filled
        filled: bool,
    },
    /// A message that is sent when a task update occurs
    TaskUpdate {
        /// The updated task
        task: ApiTask,
    },
}
