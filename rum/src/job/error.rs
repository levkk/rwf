use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("job error")]
    JobError,

    #[error("job serialization error: {0}")]
    JobSerializationError(serde_json::Error),

    #[error("job database error: {0}")]
    DatabaseError(crate::model::error::Error),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::JobSerializationError(err)
    }
}

impl From<crate::model::error::Error> for Error {
    fn from(err: crate::model::error::Error) -> Error {
        Error::DatabaseError(err)
    }
}
