use super::{ConnectionGuard, Error};

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
    pub async fn new(connection: ConnectionGuard) -> Result<Self, Error> {
        connection.query("BEGIN", &[]).await?;

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
        self.connection.query("COMMIT", &[]).await?;

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
    type Target = tokio_postgres::Client;

    fn deref(&self) -> &Self::Target {
        self.connection.deref()
    }
}
