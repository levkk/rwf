use crate::controller::auth::SessionId;
use crate::http::websocket::Message;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;
use tokio::sync::broadcast::{channel, error::SendError, Receiver, Sender};

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    SendError(#[from] SendError<Message>),
}

static MESSAGES: Lazy<Messages> = Lazy::new(|| Messages::new());

pub fn get_comms() -> &'static Messages {
    &MESSAGES
}

struct Websocket {
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

impl Clone for Websocket {
    fn clone(&self) -> Self {
        Websocket {
            sender: self.sender.clone(),
            receiver: self.receiver.resubscribe(),
        }
    }
}

impl Websocket {
    fn new() -> Self {
        let (sender, receiver) = channel(1024);
        Self { sender, receiver }
    }

    fn receiver(&self) -> Receiver<Message> {
        self.receiver.resubscribe()
    }

    fn sender(&self) -> Sender<Message> {
        self.sender.clone()
    }

    pub fn send(&self, message: Message) -> Result<usize, Error> {
        Ok(self.sender.send(message)?)
    }
}

pub struct Messages {
    websocket: Arc<Mutex<HashMap<SessionId, Websocket>>>,
}

impl Messages {
    pub fn new() -> Self {
        Self {
            websocket: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn websocket_disconnect(&self, session_id: &SessionId) {
        self.websocket.lock().remove(session_id);
    }

    pub fn websocket_connected(&self, session_id: &SessionId) -> bool {
        self.websocket.lock().get(session_id).is_some()
    }

    pub fn websocket_receiver(&self, session_id: &SessionId) -> Receiver<Message> {
        let mut guard = self.websocket.lock();
        let entry = guard
            .entry(session_id.clone())
            .or_insert_with(Websocket::new);
        entry.receiver()
    }

    pub fn websocket_sender(&self, session_id: &SessionId) -> WebsocketSender {
        let mut guard = self.websocket.lock();
        let entry = guard
            .entry(session_id.clone())
            .or_insert_with(Websocket::new);
        WebsocketSender {
            sender: entry.sender(),
        }
    }
}

#[derive(Debug)]
pub struct WebsocketSender {
    sender: Sender<Message>,
}

impl WebsocketSender {
    pub fn send(&self, message: Message) -> Result<usize, Error> {
        Ok(self.sender.send(message)?)
    }
}

impl std::ops::Deref for WebsocketSender {
    type Target = Sender<Message>;

    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}
