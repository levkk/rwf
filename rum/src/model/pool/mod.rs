use tokio::sync::Notify;
use tokio::time::{timeout, Duration};

use tokio_postgres::Client;

use parking_lot::Mutex;

use std::collections::VecDeque;
use std::sync::Arc;

use std::ops::Deref;

pub mod connection;
use super::Error;
pub use connection::Connection;

/// Smart pointer that automatically checks in the connection
/// back into the pool when the connection is dropped.
pub struct ConnectionGuard {
    connection: Option<Connection>,
    pool: Pool,
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
        }
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let connection = self.connection.take().unwrap();
        self.pool.checkin(connection);
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

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pool_size: usize,
    checkout_timeout: Duration,
}

impl PoolConfig {
    fn local() -> Self {
        Self {
            pool_size: 1,
            checkout_timeout: Duration::from_secs(1),
        }
    }
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            pool_size: 10,
            checkout_timeout: Duration::from_secs(5),
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
    /// * `pool_size` - Maximum number of connections.
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

    async fn get_internal(&self) -> Result<ConnectionGuard, Error> {
        loop {
            let mut inner = self.inner.lock();

            while !inner.connections.is_empty() {
                let candidate = inner.connections.pop_back();

                if let Some(candidate) = candidate {
                    if !candidate.bad() {
                        return Ok(ConnectionGuard::new(candidate, self.clone()));
                    }
                }
            }

            if self.config.pool_size > inner.expected {
                let connection = Connection::new(&self.database_url).await?;
                inner.expected += 1;

                return Ok(ConnectionGuard::new(connection, self.clone()));
            } else {
                self.checkin_notify.notified().await;
            }
        }
    }

    fn checkin(&self, connection: Connection) {
        let mut inner = self.inner.lock();
        if !connection.bad() {
            inner.connections.push_back(connection);
        } else {
            inner.expected -= 1;
        }

        self.checkin_notify.notify_one();
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
