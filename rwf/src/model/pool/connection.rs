//! Single `tokio_postgres` connection manager.

use tokio::select;
use tokio::sync::Notify;
use tokio::task::spawn;

use tokio_postgres::tls::NoTls;
use tokio_postgres::{types::ToSql, Client, Row, Statement};

use std::collections::HashMap;

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

    /// Execute the query against the database, preparing it if we haven't seen it before
    /// on this connection.
    pub async fn query_cached(
        &mut self,
        query: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<Row>, Error> {
        let statement = if let Some(statement) = self.cache.get(query) {
            statement
        } else {
            let statement = self.client().prepare(query).await?;
            self.cache.insert(query.to_string(), statement);
            &self.cache[query]
        };

        match self.client().query(statement, &params).await {
            Ok(rows) => Ok(rows),
            Err(err) => {
                // If schema changed, we better close this connection entirely
                // than evicting prepared statements one by one.
                // TODO: find and use the error code instead of using the English
                // error message which will be translated on databases running in other locales.
                if let Some(db_error) = err.as_db_error() {
                    if db_error.message() == "cached plan must not change result type" {
                        self.inner.bad.store(true, Ordering::Relaxed);
                    }
                }
                Err(Error::DatabaseError(err))
            }
        }
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

    pub fn client(&self) -> &Client {
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
