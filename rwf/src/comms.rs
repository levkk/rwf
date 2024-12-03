//! Communication channels between clients and servers.
//!
//! Currenty used for sending messages to clients via WebSocket connections.
//!
//! On the roadmap:
//!
//! * Send messages between clients connected to different Rwf servers
//! * ORM-triggered events, e.g. callbacks
use crate::controller::auth::SessionId;
use crate::http::websocket::Message;
use crate::http::ToMessage;
use crate::model::{Model, Value};

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;
use tokio::sync::broadcast::{channel, error::SendError, Receiver, Sender};
use tracing::debug;

/// Error returned by comms.
#[derive(Error, Debug)]
pub enum Error {
    /// Error sending message through Tokio channel.
    #[error("{0}")]
    SendError(#[from] SendError<Message>),
}

static MESSAGES: Lazy<Messages> = Lazy::new(|| Messages::new());
static DEFAULT_TOPIC: &str = "default";

fn get_comms() -> &'static Messages {
    &MESSAGES
}

struct Websocket {
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    topic: String,
}

impl Clone for Websocket {
    fn clone(&self) -> Self {
        Websocket {
            sender: self.sender.clone(),
            receiver: self.receiver.resubscribe(),
            topic: self.topic.clone(),
        }
    }
}

impl Websocket {
    fn new() -> Self {
        let (sender, receiver) = channel(1024);
        Self {
            sender,
            receiver,
            topic: DEFAULT_TOPIC.to_string(),
        }
    }

    fn receiver(&self) -> Receiver<Message> {
        self.receiver.resubscribe()
    }

    fn sender(&self) -> Sender<Message> {
        self.sender.clone()
    }
}

/// Global messages channel.
pub struct Messages {
    websocket: Arc<Mutex<HashMap<SessionId, Websocket>>>,
}

impl Messages {
    /// Create new messages channel.
    pub fn new() -> Self {
        Self {
            websocket: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn websocket_disconnect(&self, session_id: &SessionId) {
        debug!("websocket session \"{:?}\" closed", session_id);
        self.websocket.lock().remove(session_id);
    }

    /// Check that a session has an active WebSocket connection.
    pub fn websocket_connected(&self, session_id: &SessionId) -> bool {
        self.websocket.lock().get(session_id).is_some()
    }

    /// Get a websocket message receiver. All messages sent from clients will be sent to the receiver.
    pub fn websocket_receiver(&self, session_id: &SessionId, _topic: &str) -> WebsocketReceiver {
        let mut guard = self.websocket.lock();
        let entry = guard
            .entry(session_id.clone())
            .or_insert_with(Websocket::new);

        WebsocketReceiver {
            receiver: Some(entry.receiver()),
            sender: entry.sender(),
            session_id: session_id.clone(),
        }
    }

    /// Get a websocket message sender. This allows to send messages to all websocket connections
    /// that this session has.
    pub fn websocket_sender(&self, session_id: &SessionId, _topic: &str) -> WebsocketSender {
        let mut guard = self.websocket.lock();
        let entry = guard
            .entry(session_id.clone())
            .or_insert_with(Websocket::new);
        WebsocketSender {
            sender: entry.sender(),
        }
    }

    /// Get a websocket message sender that will send messages to all _other_ sessions.
    pub fn websocket_broadcast(&self, session_id: &SessionId, _topic: &str) -> Broadcast {
        let guard = self.websocket.lock();
        let entries = guard
            .iter()
            .filter(|(id, _)| *session_id != **id)
            .map(|(_, websocket)| websocket.clone())
            .collect::<Vec<_>>();

        Broadcast { everyone: entries }
    }

    /// Get a websocket message sender that will send messages to _everyone_ connected.
    pub fn websocket_notify(&self, _topic: &str) -> Broadcast {
        let guard = self.websocket.lock();
        let entries = guard
            .iter()
            .map(|(_, websocket)| websocket.clone())
            .collect::<Vec<_>>();

        Broadcast { everyone: entries }
    }
}

/// WebSocket message sender.
#[derive(Debug)]
pub struct WebsocketSender {
    sender: Sender<Message>,
}

impl WebsocketSender {
    /// Send a message via WebSocket connection.
    pub fn send(&self, message: impl ToMessage) -> Result<usize, Error> {
        Ok(self.sender.send(message.to_message())?)
    }
}

impl std::ops::Deref for WebsocketSender {
    type Target = Sender<Message>;

    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

/// Receiver for WebSocket messages.
///
/// Once this receiver is created, all subsequent messages will be sent to this
/// receiver as well as all others.
#[derive(Debug)]
pub struct WebsocketReceiver {
    receiver: Option<Receiver<Message>>,
    sender: Sender<Message>,
    session_id: SessionId,
}

impl WebsocketReceiver {
    /// Get the session ID for this WebSocket receiver.
    pub fn session_id(&self) -> &SessionId {
        &self.session_id
    }
}

impl std::ops::Deref for WebsocketReceiver {
    type Target = Receiver<Message>;

    fn deref(&self) -> &Self::Target {
        self.receiver.as_ref().unwrap()
    }
}

impl std::ops::DerefMut for WebsocketReceiver {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.receiver.as_mut().unwrap()
    }
}

impl Drop for WebsocketReceiver {
    fn drop(&mut self) {
        drop(self.receiver.take());
        if self.sender.receiver_count() == 1 {
            get_comms().websocket_disconnect(&self.session_id);
        }
    }
}

/// Send messages to every single connected
/// WebSocket session.
pub struct Broadcast {
    everyone: Vec<Websocket>,
}

impl Broadcast {
    /// Send a message to all connected sessions.
    pub fn send(&self, message: impl ToMessage) -> Result<(), Error> {
        for socket in &self.everyone {
            socket.sender.send(message.clone().to_message())?;
        }

        Ok(())
    }
}

/// Convert an object into a session.
///
/// If a model is passed in, the `id` field is used.
pub trait IntoSessionId {
    /// Convert a struct to a session identifier.
    fn into_session_id(self) -> SessionId;
}

impl IntoSessionId for &SessionId {
    fn into_session_id(self) -> SessionId {
        self.clone()
    }
}

impl IntoSessionId for SessionId {
    fn into_session_id(self) -> SessionId {
        self
    }
}

impl IntoSessionId for &i64 {
    fn into_session_id(self) -> SessionId {
        SessionId::Authenticated(*self)
    }
}

impl IntoSessionId for i64 {
    fn into_session_id(self) -> SessionId {
        SessionId::Authenticated(self)
    }
}

impl<T: Model> IntoSessionId for &T {
    fn into_session_id(self) -> SessionId {
        match self.id() {
            Value::Optional(user_id) => match *user_id {
                Some(Value::Integer(user_id)) => SessionId::Authenticated(user_id),
                _ => panic!("session id cannot be extrated"),
            },
            _ => panic!("session id cannot be extracted"),
        }
    }
}

/// App-wide communications using WebSockets.
pub struct Comms;

impl Comms {
    /// Get a handle for a WebSocket connection for a session.
    ///
    /// Allows sending WebSocket messages to all connections with this session.
    pub fn websocket(session: impl IntoSessionId) -> WebsocketSender {
        let session_id = session.into_session_id();
        get_comms().websocket_sender(&session_id, DEFAULT_TOPIC)
    }

    /// Get a handle for a WebSocket connection _receiver_ for a session.
    ///
    /// Allows listening for WebSocket messages sent by clients (browsers)
    /// connected with this session.
    pub fn receiver(session_id: impl IntoSessionId) -> WebsocketReceiver {
        let session_id = session_id.into_session_id();
        get_comms().websocket_receiver(&session_id, DEFAULT_TOPIC)
    }

    /// Get a broadcast handle for a WebSocket message to everyone else except
    /// the session sending this message.
    pub fn broadcast(session_id: impl IntoSessionId) -> Broadcast {
        let session_id = session_id.into_session_id();
        get_comms().websocket_broadcast(&session_id, DEFAULT_TOPIC)
    }

    /// Used for dev server notifications (sent to every connected session).
    pub fn notify() -> Broadcast {
        get_comms().websocket_notify(DEFAULT_TOPIC)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::{FromRow, ToValue, Value};

    #[test]
    fn test_websocket_sender() {
        let session = SessionId::Authenticated(5);
        let websocket = Comms::websocket(&session);
        websocket.send(Message::Text("test".into())).unwrap();

        #[derive(Clone)]
        struct User {
            id: Option<i64>,
        }

        impl Model for User {
            fn table_name() -> &'static str {
                "users"
            }

            fn foreign_key() -> &'static str {
                "user_id"
            }

            fn column_names() -> &'static [&'static str] {
                &[]
            }

            fn values(&self) -> Vec<Value> {
                vec![]
            }

            fn id(&self) -> Value {
                self.id.to_value()
            }
        }

        impl FromRow for User {
            fn from_row(_row: tokio_postgres::Row) -> Result<Self, crate::model::Error>
            where
                Self: Sized,
            {
                unimplemented!()
            }
        }

        let user = User { id: Some(5) };
        let websocket = Comms::websocket(&user);
        websocket.send(Message::Text("test2".into())).unwrap();
    }
}
