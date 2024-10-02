use crate::controller::auth::UserId;
use crate::http::websocket::Message;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::broadcast::{channel, Receiver, Sender};

static MESSAGES: Lazy<Messages> = Lazy::new(|| Messages::new());

pub fn get_comms() -> &'static Messages {
    &MESSAGES
}

struct WebsocketComms {
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

impl Clone for WebsocketComms {
    fn clone(&self) -> Self {
        WebsocketComms {
            sender: self.sender.clone(),
            receiver: self.receiver.resubscribe(),
        }
    }
}

impl WebsocketComms {
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
}

pub struct Messages {
    websocket: Arc<Mutex<HashMap<UserId, WebsocketComms>>>,
}

impl Messages {
    pub fn new() -> Self {
        Self {
            websocket: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn receiver(&self, user_id: &UserId) -> Receiver<Message> {
        let mut guard = self.websocket.lock();
        let entry = guard
            .entry(user_id.clone())
            .or_insert_with(WebsocketComms::new);
        entry.receiver()
    }

    pub fn sender(&self, user_id: &UserId) -> Sender<Message> {
        let mut guard = self.websocket.lock();
        let entry = guard
            .entry(user_id.clone())
            .or_insert_with(WebsocketComms::new);
        entry.sender()
    }
}
