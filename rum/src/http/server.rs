use super::{Error, Handler, Request};

use std::sync::Arc;
use tokio::net::TcpListener;

pub struct Server {
    handlers: Arc<Vec<Handler>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            handlers: Arc::new(vec![]),
        }
    }

    pub async fn launch(self) -> Result<(), Error> {
        let listener = TcpListener::bind("0.0.0.0:8000").await?;

        loop {
            let (mut stream, _peer_addr) = listener.accept().await?;
            let handlers = self.handlers.clone();

            // tokio::spawn(async move {
            let request = Request::read(&mut stream).await.expect("request");
            for handler in handlers.iter() {
                if request.path().matches(handler.path()) {}
            }
            // });
        }
    }
}
