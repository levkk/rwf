use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("job error")]
    JobError,

    #[error("job serialization error: {0}")]
    JobSerializationError(serde_json::Error),

    #[error("job database error: {0}")]
    DatabaseError(crate::model::error::Error),

    #[error("job retry")]
    Retry,

    #[error("tokio error: {0}")]
    WorkerError(#[from] tokio::task::JoinError),

    #[error("job error: {0}")]
    Unknown(String),

    #[error("comms error: {0}")]
    CommsError(#[from] crate::comms::Error),

    #[error("specified cron schedule is not valid")]
    CronValueError,
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

impl From<tokio_postgres::Error> for Error {
    fn from(err: tokio_postgres::Error) -> Error {
        Error::DatabaseError(err.into())
    }
}
