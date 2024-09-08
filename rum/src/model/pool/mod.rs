use tokio::sync::Notify;
use tokio::task::spawn;
use tokio::time::{timeout, Duration};

use tokio_postgres::Client;

use parking_lot::Mutex;

use std::collections::VecDeque;
use std::future::Future;
use std::ops::Deref;
use std::sync::Arc;

use once_cell::sync::OnceCell;

pub mod connection;
pub mod transaction;

use super::Error;

pub use connection::Connection;
pub use transaction::Transaction;

static POOL: OnceCell<Pool> = OnceCell::new();

pub fn get_pool() -> Pool {
    POOL.get_or_init(|| Pool::new_local()).clone()
}

pub async fn get_connection() -> Result<ConnectionGuard, Error> {
    get_pool().get().await
}

pub async fn start_transaction() -> Result<Transaction, Error> {
    get_pool().begin().await
}

/// Smart pointer that automatically checks in the connection
/// back into the pool when the connection is dropped.
pub struct ConnectionGuard {
    connection: Option<Connection>,
    pool: Pool,
    rollback: bool,
}

impl ConnectionGuard {
    /// Create new connection guard.
    ///
    /// # Arguments
    ///
    /// * `connection` - Connection to Postgres.
    /// * `pool` - Pool from which the connection was acquired.
    ///
    pub fn new(connection: Connection, pool: Pool) -> Self {
        ConnectionGuard {
            connection: Some(connection),
            pool,
            rollback: false,
        }
    }

    fn rollback(&mut self) {
        self.rollback = true;
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        if let Some(connection) = self.connection.take() {
            if self.rollback {
                let pool = self.pool.clone();
                spawn(async move {
                    pool.checkin_rollback(connection).await;
                });
            } else {
                self.pool.checkin(connection, false);
            }
        }
    }
}

impl Deref for ConnectionGuard {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        self.connection.as_ref().unwrap()
    }
}

#[derive(Debug)]
struct PoolInner {
    connections: VecDeque<Connection>,
    expected: usize,
}

/// Connection pool configuration options.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of open PostgreSQL connections in the pool.
    pub pool_size: usize,

    /// Maximum time to wait for a connection to be created or returned into the pool
    /// by another caller.
    pub checkout_timeout: Duration,

    /// Maximum time a connection remains open and available while not in use.
    pub idle_timeout: Duration,
}

impl PoolConfig {
    fn local() -> Self {
        Self {
            pool_size: 1,
            checkout_timeout: Duration::from_secs(1),
            idle_timeout: Duration::from_secs(3600),
        }
    }
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            pool_size: 10,
            checkout_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(3600),
        }
    }
}

/// Connection pool that automatically manages connections.
#[derive(Debug, Clone)]
pub struct Pool {
    inner: Arc<Mutex<PoolInner>>,
    checkin_notify: Arc<Notify>,
    database_url: String,
    config: PoolConfig,
}

impl Pool {
    /// Create new connection pool.
    ///
    /// # Arguments
    ///
    /// * `database_url` - Postgres-style connection URL.
    /// * `pool_config` - Pool configuration options.
    ///
    pub fn new(database_url: &str, config: PoolConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(PoolInner {
                connections: VecDeque::new(),
                expected: 0,
            })),
            checkin_notify: Arc::new(Notify::new()),
            database_url: database_url.to_string(),
            config,
        }
    }

    /// Create new connection pool to a local Postgres instance.
    ///
    /// The driver will likely connect using the UNIX socket.
    ///
    /// # Arguments
    ///
    /// * `pool_size` - Maximum number of connections.
    ///
    pub fn new_local() -> Self {
        let user = std::env::var("USER").unwrap_or("postgres".to_string());
        Self::new(
            &format!("postgresql://{}@localhost", user),
            PoolConfig::local(),
        )
    }

    /// Get a connection from the pool or wait until one is available.
    pub async fn get(&self) -> Result<ConnectionGuard, Error> {
        match timeout(self.config.checkout_timeout, self.get_internal()).await {
            Ok(result) => result,
            Err(_) => Err(Error::PoolTimeout),
        }
    }

    /// See [`Pool::transaction`]
    pub async fn begin(&self) -> Result<Transaction, Error> {
        let connection = self.get().await?;
        Ok(Transaction::new(connection).await?)
    }

    /// Start a new transaction.
    ///
    /// The transaction should be manually committed with [`Transaction::commit`]
    /// otherwise it will be automatically rolled back.
    pub async fn transaction(&self) -> Result<Transaction, Error> {
        self.begin().await
    }

    pub async fn with_transaction<Fut, R>(
        &self,
        f: impl FnOnce(Transaction) -> Fut,
    ) -> Result<R, Error>
    where
        Fut: Future<Output = Result<R, Error>>,
    {
        let transaction = self.begin().await?;
        let result = f(transaction).await?;
        // transaction.commit().await?;
        Ok(result)
    }

    pub async fn with_connection<Fut, R>(
        &self,
        f: impl FnOnce(ConnectionGuard) -> Fut,
    ) -> Result<R, Error>
    where
        Fut: Future<Output = Result<R, Error>>,
    {
        let connection = self.get().await?;
        f(connection).await
    }

    // Get a connection from the pool or create a new one if allowed.
    async fn get_internal(&self) -> Result<ConnectionGuard, Error> {
        loop {
            {
                let mut inner = self.inner.lock();

                while !inner.connections.is_empty() {
                    let candidate = inner.connections.pop_back();

                    if let Some(candidate) = candidate {
                        if !candidate.bad() {
                            return Ok(ConnectionGuard::new(candidate, self.clone()));
                        }
                    }
                }
            }

            let need_more = {
                let mut inner = self.inner.lock();
                let need_more = self.config.pool_size > inner.expected;

                if need_more {
                    inner.expected += 1;
                }

                need_more
            };

            if need_more {
                match Connection::new(&self.database_url).await {
                    Ok(connection) => return Ok(ConnectionGuard::new(connection, self.clone())),
                    Err(err) => {
                        {
                            let mut inner = self.inner.lock();
                            inner.expected -= 1;
                        }
                        return Err(err);
                    }
                }
            } else {
                self.checkin_notify.notified().await;
            }
        }
    }

    fn checkin(&self, connection: Connection, drop: bool) {
        let mut inner = self.inner.lock();
        if !connection.bad() && !drop {
            inner.connections.push_back(connection);
        } else {
            inner.expected -= 1;
        }

        self.checkin_notify.notify_one();
    }

    async fn checkin_rollback(&self, connection: Connection) {
        match connection.execute("ROLLBACK", &[]).await {
            Ok(_) => {
                tracing::debug!("ROLLBACK");
                self.checkin(connection, false)
            }
            Err(err) => {
                tracing::error!("auto rollback failed: {:?}", err);
                self.checkin(connection, true)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_pool() -> Result<(), Error> {
        let pool = Pool::new_local();
        let conn = pool.get().await?;
        let row = conn.query("SELECT 1", &[]).await?;

        assert_eq!(row.len(), 1);

        assert!(pool.get().await.is_err());

        drop(conn);
        assert!(pool.get().await.is_ok());

        Ok(())
    }
}
