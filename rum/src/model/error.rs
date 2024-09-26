use thiserror::Error;

use super::Value;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown error: {0}")]
    Unknown(String),

    #[error("database error: {0}")]
    DatabaseError(tokio_postgres::Error),

    #[error("query error: {0}, query: {1}")]
    QueryError(String, String),

    #[error("ORM serialization error: {0:?}")]
    OrmSerializationError(Value),

    #[error("{0}: {1}")]
    ValueError(&'static str, String),

    #[error("pool timeout")]
    PoolTimeout,

    #[error("pool not configured")]
    PoolNotConfigured,

    #[error("record not found")]
    RecordNotFound,

    #[error("unknown token in template: {0}")]
    UnknownToken(String),

    #[error("template syntax error: {0}")]
    SyntaxError(String),

    #[error("migration error: {0}")]
    MigrationError(String),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

impl Error {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(error: tokio_postgres::Error) -> Error {
        Error::DatabaseError(error)
    }
}
