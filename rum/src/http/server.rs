use super::{Error, Request, Response, Route};
use std::future::Future;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

use std::time::Instant;
use std::sync::Arc;
use tracing::{debug, info};

pub struct Server {
}

impl Server {
    pub fn new() -> Self {
        todo!()
    }

    pub async fn launch(self) -> Result<(), Error> {
        let mut listener = TcpListener::bind("0.0.0.0:8000").await?;

        loop {
            let (mut stream, peer_addr) = listener.accept().await?;
        }

        Ok(())
    }
}
