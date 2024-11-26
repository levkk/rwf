//! Database connection pool.
//!
//! The pool is responsible for creating and maintaining connections to the database. Clients can request a connection
//! by calling [`Pool::connection`] and awaiting the result. If a connection isn't available within a configurable
//! amount of time, an error is returned.
//!
//! The pool manages transactions by starting one with [`Pool::begin`]. The returned struct can be used to execute all
//! ORM methods, just like a regular connection. When the transaction is finished, the caller should execute
//! the [`Transaction::commit`] method and await the result. If the transaction reference is dropped
//! with an uncomitted transaction, it will be rolled back automatically.
//!
//! This implementation uses FIFO to increase connection re-use.
//!
//! ## Get a connection
//!
//! ```ignore
//! let conn = Pool::connection().await?;
//! ```
//!
//! ## Start a transaction
//!
//! ```ignore
//! let transcation = Pool::begin().await?;
//! // execute statements
//! transaction.commit().await?;
//! ```
use tokio::select;
use tokio::sync::Notify;
use tokio::task::spawn;
use tokio::time::{sleep, timeout, Duration};

use parking_lot::Mutex;

use std::collections::VecDeque;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Instant;

use once_cell::sync::OnceCell;

use crate::config::get_config;

pub mod connection;
pub mod transaction;

use super::Error;

pub use connection::Connection;
pub use transaction::Transaction;

static POOL: OnceCell<Pool> = OnceCell::new();

/// Get the connection pool.
///
/// Use [`Pool::pool`] instead.
pub fn get_pool() -> Pool {
    POOL.get_or_init(|| Pool::from_env()).clone()
}

/// Get a connection from the pool.
///
/// Use [`Pool::connection`] instead.
pub async fn get_connection() -> Result<ConnectionGuard, Error> {
    get_pool().get().await
}

/// Start a transaction.
///
/// Use [`Pool::begin`] instead.
pub async fn start_transaction() -> Result<Transaction, Error> {
    get_pool().transaction().await
}

/// Smart pointer that automatically checks in the connection
/// back into the pool when the connection is dropped.
pub struct ConnectionGuard {
    connection: Option<Connection>,
    pool: Pool,
    rollback: bool,
    leaked: bool,
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
            leaked: false,
        }
    }

    fn rollback(&mut self) {
        self.rollback = true;
    }

    /// Get a reference to the underlying database connection.
    pub fn connection(&self) -> &Connection {
        self.connection.as_ref().unwrap()
    }

    /// Get a mutable reference to the underlying database connection.
    pub fn connection_mut(&mut self) -> &mut Connection {
        self.connection.as_mut().unwrap()
    }

    /// Take this connection from the pool forever. The pool will pretend
    /// like this connection never existed.
    ///
    /// ### Note
    ///
    /// Leaking too many connections can increase the number of open connections
    /// to your database beyond acceptable limits.
    pub fn leak(&mut self) {
        if !self.leaked {
            self.pool.leak(self.connection());
            self.leaked = true;
        }
    }
}

impl Drop for ConnectionGuard {
    /// Return the connection to the pool, automatically
    /// rolling back any unfinished transaction.
    fn drop(&mut self) {
        if self.leaked {
            return;
        }

        if let Some(mut connection) = self.connection.take() {
            connection.used();

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
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        self.connection.as_ref().unwrap()
    }
}

impl DerefMut for ConnectionGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.connection.as_mut().unwrap()
    }
}

#[derive(Debug)]
struct PoolInner {
    connections: VecDeque<Connection>,

    /// Number of connections the pool has idle
    /// and checked out by users.
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
#[derive(Debug)]
pub struct Pool {
    inner: Arc<Mutex<PoolInner>>,
    checkin_notify: Arc<Notify>,
    database_url: String,
    config: PoolConfig,
    shutdown: Arc<Notify>,
    ref_count: Arc<AtomicUsize>,
}

impl Clone for Pool {
    fn clone(&self) -> Self {
        let clone = Self {
            inner: self.inner.clone(),
            checkin_notify: self.checkin_notify.clone(),
            database_url: self.database_url.clone(),
            config: self.config.clone(),
            shutdown: self.shutdown.clone(),
            ref_count: self.ref_count.clone(),
        };

        self.ref_count.fetch_add(1, Ordering::SeqCst);

        clone
    }
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
        let pool = Self {
            inner: Arc::new(Mutex::new(PoolInner {
                connections: VecDeque::new(),
                expected: 0,
            })),
            checkin_notify: Arc::new(Notify::new()),
            database_url: database_url.to_string(),
            config,
            shutdown: Arc::new(Notify::new()),
            ref_count: Arc::new(AtomicUsize::new(1)),
        };

        let maintenance = pool.clone();
        tokio::spawn(async move {
            loop {
                select! {
                    _ = sleep(Duration::from_secs(1)) => {
                        maintenance.maintenance();
                    }

                    _ = maintenance.shutdown.notified() => {
                        break;
                    }
                }
            }
        });

        pool
    }

    /// Create new connection pool to a local Postgres instance.
    ///
    /// The driver will likely connect using the UNIX socket.
    ///
    /// # Arguments
    ///
    /// * `pool_size` - Maximum number of connections.
    ///
    pub fn from_env() -> Self {
        let config = get_config().database.clone();
        let database_url = config.database_url();
        Self::new(
            &database_url,
            PoolConfig {
                pool_size: config.pool_size,
                idle_timeout: config.idle_timeout().unsigned_abs(),
                checkout_timeout: config.checkout_timeout().unsigned_abs(),
            },
        )
    }

    /// Get a connection from the pool or wait until one is available.
    pub async fn get(&self) -> Result<ConnectionGuard, Error> {
        match timeout(self.config.checkout_timeout, self.get_internal()).await {
            Ok(result) => result,
            Err(_) => {
                // self.inner.lock().expected -= 1;
                Err(Error::PoolTimeout)
            }
        }
    }

    pub fn pool() -> Self {
        get_pool()
    }

    pub async fn connection() -> Result<ConnectionGuard, Error> {
        let pool = get_pool();
        pool.get().await
    }

    /// See [`Pool::transaction`]
    pub async fn begin() -> Result<Transaction, Error> {
        let pool = get_pool();
        pool.transaction().await
    }

    /// Start a new transaction.
    ///
    /// The transaction should be manually committed with [`Transaction::commit`]
    /// otherwise it will be automatically rolled back.
    pub async fn transaction(&self) -> Result<Transaction, Error> {
        let connection = self.get().await?;
        Ok(Transaction::new(connection).await?)
    }

    pub async fn with_transaction<Fut, R>(
        &self,
        f: impl FnOnce(Transaction) -> Fut,
    ) -> Result<R, Error>
    where
        Fut: Future<Output = Result<R, Error>>,
    {
        let transaction = self.transaction().await?;
        let result = f(transaction).await?;
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
                        } else {
                            inner.expected -= 1;
                        }
                        // Drop (close) all bad connections.
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
        {
            let mut inner = self.inner.lock();
            if !connection.bad() && !drop {
                inner.connections.push_back(connection);
            } else {
                inner.expected -= 1;
            }
        }

        self.checkin_notify.notify_one();
    }

    /// Take the connection from the pool forever.
    ///
    /// The caller is responsible for closing the connection. The pool
    /// will pretend like this connection never existed.
    fn leak(&self, _connection: &Connection) {
        self.inner.lock().expected -= 1;
    }

    fn maintenance(&self) {
        let now = Instant::now();
        let mut inner = self.inner.lock();

        let before = inner.connections.len();
        inner.connections.retain(|c| {
            let age = now.duration_since(c.last_used());
            let too_old = age > self.config.idle_timeout;
            !c.bad() && !too_old
        });
        let removed = before - inner.connections.len();
        inner.expected -= removed;
    }

    async fn checkin_rollback(&self, mut connection: Connection) {
        match connection.query_cached("ROLLBACK", &[]).await {
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

impl Drop for Pool {
    fn drop(&mut self) {
        let ref_count = self.ref_count.fetch_sub(1, Ordering::SeqCst);

        if ref_count == 1 {
            self.shutdown.notify_one();
        }
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use super::*;

    #[tokio::test]
    async fn test_pool() -> Result<(), Error> {
        env::set_var("RWF_DATABASE_CHECKOUT_TIMEOUT", "500");

        let pool = Pool::from_env();
        let conn = pool.get().await?;
        let row = conn.client().query("SELECT 1", &[]).await?;

        assert_eq!(row.len(), 1);
        assert_eq!(pool.inner.lock().expected, 1);

        let mut consume = vec![];

        for i in 0..9 {
            consume.push(pool.get().await);
            let expected = { pool.inner.lock().expected };
            let conns = { pool.inner.lock().connections.len() };
            assert_eq!(expected, 1 + i + 1);
            assert_eq!(conns, 0); // All are checked out.
        }

        assert!(pool.get().await.is_err());

        assert_eq!(pool.inner.lock().connections.len(), 0);
        drop(conn); // Conn returned to pool

        let mut conn = pool.get().await?;
        assert!(pool.get().await.is_err());

        assert_eq!(pool.inner.lock().expected, 10);

        conn.leak();
        assert_eq!(pool.inner.lock().expected, 9);
        assert!(pool.get().await.is_ok());

        assert_eq!(pool.inner.lock().expected, 10);
        assert_eq!(pool.inner.lock().connections.len(), 1);
        consume.clear();
        assert_eq!(pool.inner.lock().connections.len(), 10);
        assert_eq!(pool.inner.lock().expected, 10);

        Ok(())
    }

    #[tokio::test]
    async fn test_bad_pool() {
        env::set_var("RWF_DATABASE_CHECKOUT_TIMEOUT", "500");

        let pool = Pool::from_env();
        assert_eq!(pool.inner.lock().expected, 0);

        {
            let conn = pool.get().await.unwrap();
            assert_eq!(pool.inner.lock().expected, 1);
            conn.close();
        }

        assert_eq!(pool.inner.lock().expected, 0);

        {
            let _conn = pool.get().await.unwrap();
            assert_eq!(pool.inner.lock().expected, 1);
        }

        assert_eq!(pool.inner.lock().expected, 1);
        let _conn = pool.get().await.unwrap();
        let _conn = pool.get().await.unwrap();
        assert_eq!(pool.inner.lock().expected, 2);
    }
}
