use super::{ConnectionGuard, Error};

pub struct Transaction {
    connection: ConnectionGuard,
    rollback: bool,
}

impl Transaction {
    pub fn new(connection: ConnectionGuard) -> Self {
        Self {
            connection,
            rollback: true,
        }
    }

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
