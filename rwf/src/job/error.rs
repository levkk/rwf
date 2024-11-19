//! Errors returned by the job queue.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    /// General error.
    #[error("job error")]
    JobError,

    /// Couldn't (de)serialize job arguments.
    #[error("job serialization error: {0}")]
    JobSerializationError(serde_json::Error),

    /// The ORM returned an error.
    #[error("job database error: {0}")]
    DatabaseError(crate::model::error::Error),

    /// Job should be retried.
    #[error("job retry")]
    Retry,

    /// A worker thread blew up. This indicates a panic
    /// in the job code.
    #[error("tokio error: {0}")]
    WorkerError(#[from] tokio::task::JoinError),

    /// Something happened, we don't know.
    #[error("job error: {0}")]
    Unknown(String),

    /// Error returned from the communication system.
    #[error("comms error: {0}")]
    CommsError(#[from] crate::comms::Error),

    /// Cron value specified for the schedule isn't valid.
    #[error("specified cron schedule is not valid")]
    CronValueError,

    #[error("lost connection to cron database")]
    CronConnectionError,
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
