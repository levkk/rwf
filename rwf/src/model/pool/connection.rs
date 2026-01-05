//! Wraps [`tokio_postgres::Client`].

use time::Duration;
use tokio::select;
use tokio::sync::Notify;
use tokio::task::spawn;

use tokio_postgres::{connect, types::ToSql, Client, Row, Statement};

use tracing::info;

use std::collections::HashMap;

use super::Error;
use crate::config::get_config;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;

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
    created_at: Instant,
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
        let config = get_config().database.tls_config()?;

        let bad = AtomicBool::new(false);
        let shutdown = Notify::new();

        let inner = Arc::new(ConnectionInner { bad, shutdown });

        let mut guard = match config {
            crate::config::DBTlsConfig::On(c) => {
                let (client, connection) = connect(database_url, c).await?;
                let client = Connection {
                    client,
                    inner: inner.clone(),
                    last_used: Instant::now(),
                    created_at: Instant::now(),
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
                client
            }
            crate::config::DBTlsConfig::Off(c) => {
                let (client, connection) = connect(database_url, c).await?;
                let client = Connection {
                    client,
                    inner: inner.clone(),
                    last_used: Instant::now(),
                    created_at: Instant::now(),
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
                client
            }
        };

        let info = guard
            .query_cached("SELECT current_database()::text, current_user::text", &[])
            .await?;

        let row = info.get(0).unwrap();
        let user: String = row.get::<_, String>(1);
        let database: String = row.get::<_, String>(0);

        info!(
            "New connection to database \"{}\" with user \"{}\" created",
            database, user
        );

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

    /// Forcibly close the connection once it's returned to the pool.
    pub fn close(&self) {
        self.inner.bad.store(true, Ordering::Relaxed);
    }

    /// Indicate the connection was last used now.
    pub fn used(&mut self) {
        self.last_used = Instant::now();
    }

    /// Get the time when this connection was last used.
    pub fn last_used(&self) -> Instant {
        self.last_used
    }

    /// Get the database driver reference to manually execute
    /// queries against the database, bypassing the connection manager.
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
        let elapsed = self.created_at.elapsed();
        let duration = Duration::try_from(elapsed).unwrap_or(Duration::seconds(0));
        info!(
            "Connection to database closed (age: {:0>2}h{:0>2}m{:0>2}s)",
            duration.whole_hours(),
            duration.whole_minutes(),
            duration.whole_seconds(),
        );
    }
}
