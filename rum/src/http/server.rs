use super::Error;

use tokio::net::TcpListener;

pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Server {}
    }

    pub async fn launch(self) -> Result<(), Error> {
        let listener = TcpListener::bind("0.0.0.0:8000").await?;

        loop {
            let (_stream, _peer_addr) = listener.accept().await?;
        }
    }
}
