//! Single `tokio_postgres` connection manager.

use tokio::select;
use tokio::sync::Notify;
use tokio::task::spawn;

use tokio_postgres::tls::NoTls;
use tokio_postgres::{types::ToSql, Client, Row, Statement};

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;

use super::Error;
use crate::model::{FromRow, Placeholders};

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
    cache: HashMap<String, Statement>,
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
            cache: HashMap::new(),
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

    pub async fn query_cached(
        &mut self,
        query: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<Row>, Error> {
        let statement = if let Some(statement) = self.cache.get(query) {
            statement
        } else {
            let statement = self.client().prepare(query).await?;
            self.cache.insert(query.to_string(), statement);
            &self.cache[query]
        };

        Ok(self.client().query(statement, &params).await?)
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
        &self.client
    }
}

impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}
