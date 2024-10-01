use super::Error;
use crate::http::websocket;
use async_trait::async_trait;
use tokio::sync::mpsc::{Receiver, Sender, channel};

use std::ops::Deref;
use std::marker::PhantomData;

#[async_trait]
pub trait Websocket: Send + Sync {
    async fn handle_message(&self, message: websocket::Message) -> Result<(), Error>;

    async fn handler(self) -> WebsocketHandler
    where
        Self: Sized + 'static,
    {
        let (tx, rx) = channel(1024);
        WebsocketHandler {
            handler: Box::new(self),
            tx,
            rx: Some(rx),
        }
    }

    async fn controller_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

pub struct WebsocketHandler {
    handler: Box<dyn Websocket>,
    tx: Sender<websocket::Message>,
    rx: Option<Receiver<websocket::Message>>,
}

impl Deref for WebsocketHandler {
    type Target = Box<dyn Websocket>;

    fn deref(&self) -> &Self::Target {
        &self.handler
    }
}

#[derive(Clone)]
pub struct WebsocketClient<T> {
    // sender: Sender<websocket::Message>,
    __marker: PhantomData<T>
}
