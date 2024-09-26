use super::{Connection, ConnectionGuard, Error};

use std::time::Instant;
use tracing::info;

/// Explicit PostgreSQL transaction.
pub struct Transaction {
    connection: ConnectionGuard,
    rollback: bool,
}

impl Transaction {
    /// Start a new transaction on the connection.
    ///
    /// The transaction is automatically rolled back if it is not committed
    /// manually using [`Transaction::commit`].
    pub async fn new(mut connection: ConnectionGuard) -> Result<Self, Error> {
        let start = Instant::now();
        connection.query_cached("BEGIN", &[]).await?;

        info!("BEGIN ({:.3} ms)", start.elapsed().as_secs_f64() * 1000.0);

        Ok(Self {
            connection,
            rollback: true,
        })
    }

    /// Commit the transaction to the database.
    ///
    /// The connection is automatically returned into the pool.
    pub async fn commit(mut self) -> Result<(), Error> {
        self.rollback = false;

        let start = Instant::now();
        self.connection.query_cached("COMMIT", &[]).await?;

        info!("COMMIT ({:.3} ms)", start.elapsed().as_secs_f64() * 1000.0);

        Ok(())
    }

    /// Rollback the transaction.
    ///
    /// The connection is automatically returned into the pool.
    pub async fn rollback(mut self) -> Result<(), Error> {
        self.rollback = false;

        let start = Instant::now();
        self.connection.query_cached("ROLLBACK", &[]).await?;

        info!(
            "ROLLBACK ({:.3} ms)",
            start.elapsed().as_secs_f64() * 1000.0
        );

        Ok(())
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if self.rollback {
            self.connection.rollback();
        }
    }
}

impl std::ops::Deref for Transaction {
    type Target = ConnectionGuard;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl std::ops::DerefMut for Transaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.connection
    }
}
