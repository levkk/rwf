//! Errors that can be returned by a controller.
//!
//! Automatic conversions exist between many standard library, tokio and template errors,
//! allowing you to use the `?` operator in controllers. For any errors that don't, you can implement
//! the `From<YourError> for Error` trait. You can also manually wrap your errors with this error, e.g. by
//! calling `Error::new(your_error)`.
use crate::http::Error as HttpError;
use thiserror::Error;

/// A controller error.
#[derive(Error, Debug)]
pub enum Error {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("database error: {0}")]
    OrmError(#[from] crate::model::Error),

    #[error("job error: {0}")]
    JobError(#[from] crate::job::Error),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("fmt error: {0}")]
    FmtError(#[from] std::fmt::Error),

    #[error("comms error: {0}")]
    CommsError(#[from] crate::comms::Error),

    #[error("view error: {0}")]
    ViewError(#[from] crate::view::Error),

    #[error("crypto error: {0}")]
    CryptoError(#[from] crate::crypto::Error),

    #[error("{0}")]
    Error(#[from] Box<dyn std::error::Error + Sync + Send>),

    #[error("http error")]
    HttpError(Box<HttpError>),

    #[error("config error: {0}")]
    Config(#[from] crate::config::Error),

    #[error("session is not set")]
    SessionMissingError,

    #[error("timeout exceeded")]
    TimeoutError(#[from] tokio::time::error::Elapsed),

    #[error("user error: {0}")]
    UserError(#[from] crate::model::user::Error),
}

impl Error {
    /// Create new error from any error implementing the standard [`std::error::Error`] trait.
    pub fn new(err: impl std::error::Error + Send + Sync + 'static) -> Error {
        Error::Error(Box::new(err))
    }
}

impl From<crate::http::Error> for Error {
    fn from(error: crate::http::Error) -> Self {
        Error::HttpError(Box::new(error))
    }
}
