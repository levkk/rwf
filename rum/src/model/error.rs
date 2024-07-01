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
