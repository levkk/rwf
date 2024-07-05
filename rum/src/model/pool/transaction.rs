use super::ConnectionGuard;

pub struct Transaction {
    connection: ConnectionGuard,
}

impl Transaction {
    pub fn new(connection: ConnectionGuard) -> Self {
        Self { connection }
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        self.connection.rollback();
    }
}

impl std::ops::Deref for Transaction {
    type Target = tokio_postgres::Client;

    fn deref(&self) -> &Self::Target {
        self.connection.deref()
    }
}
