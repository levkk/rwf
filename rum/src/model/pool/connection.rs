//! Single `tokio_postgres` connection manager.

use tokio::select;
use tokio::sync::Notify;
use tokio::task::spawn;

use tokio_postgres::tls::NoTls;
use tokio_postgres::Client;

use std::ops::Deref;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;

use super::Error;

#[derive(Debug)]
struct ConnectionInner {
    bad: AtomicBool,
    shutdown: Notify,
}

/// Wrapper around a [`tokio_postgres::Client`] that manages the connection.
#[derive(Debug)]
pub struct Connection {
    client: Client,
    inner: Arc<ConnectionInner>,
    last_used: Instant,
}

impl Connection {
    /// Create a new connection to the database.
    ///
    /// # Arguments
    ///
    /// * `database_url` - Postgres-style connection URL.
    ///
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;
        let bad = AtomicBool::new(false);
        let shutdown = Notify::new();

        let inner = Arc::new(ConnectionInner { bad, shutdown });

        let guard = Connection {
            client,
            inner: inner.clone(),
            last_used: Instant::now(),
        };

        spawn(async move {
            select! {
                error = connection => {
                    if let Err(error) = error {
                        inner.bad.store(true, Ordering::Relaxed);
                        tracing::error!("{:?}", error);
                    }
                }

                _ = inner.shutdown.notified() => {}
            }
        });

        Ok(guard)
    }

    /// Is the connection broken?
    pub fn bad(&self) -> bool {
        self.inner.bad.load(Ordering::Relaxed)
    }

    pub fn used(&mut self) {
        self.last_used = Instant::now();
    }

    pub fn last_used(&self) -> Instant {
        self.last_used
    }

    fn client(&self) -> &Client {
        &self.client
    }

    fn shutdown(&self) {
        self.inner.shutdown.notify_one();
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl Deref for Connection {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        self.client()
    }
}
