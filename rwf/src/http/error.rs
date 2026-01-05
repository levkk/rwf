//! Errors returned by the HTTP protocol implementation.

use thiserror::Error;

use super::Head;

/// Errors returned by the HTTP implementation.
#[derive(Error, Debug)]
pub enum Error {
    /// Input/output error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// The request we received is not formatted
    /// according to the HTTP specification.
    #[error("malformed request: {0}")]
    MalformedRequest(&'static str),

    /// Error encoding/decoding JSON.
    #[error("json")]
    Json(#[from] serde_json::Error),

    /// Error returned by a controller.
    #[error("{0}")]
    Controller(crate::controller::Error),

    /// Error returned by cryptographic functions.
    #[error("{0}")]
    Crypto(#[from] crate::crypto::Error),

    /// Error encoding/decoding UTF-8.
    /// All text used by Rwf must be UTF-8 encoded.
    #[error("{0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// A regex returned an error.
    #[error("{0}")]
    Regex(#[from] regex::Error),

    /// Something wrong with a time, probably out of range.
    #[error("{0}")]
    Time(time::error::ComponentRange),

    /// A required parameter is missing, e.g. from a `POST` form.
    #[error("parameter is missing")]
    MissingParameter,

    /// Something took too long.
    #[error("timeout exceeded")]
    Timeout(#[from] tokio::time::error::Elapsed),

    /// The ORM returned an error.
    #[error("database error: {0}")]
    Orm(#[from] crate::model::Error),

    /// The user isn't logged in.
    #[error("unauthorized")]
    Unauthorized,

    /// HTTP request exceeds configured size.
    #[error("content too large")]
    ContentTooLarge(Head),

    /// Model used as user doesn't have an integer id column.
    #[error("user model id is not an integer")]
    UserIdNotAnInteger,

    /// Model used as user has null id column.
    #[error("user model is is null")]
    UserIdIsNull,

    /// RustTLS
    #[error("tls error")]
    Tls(#[from] rustls::Error),

    /// PEM File
    #[error("PEM file error")]
    Pem(#[from] rustls::pki_types::pem::Error),
}

impl Error {
    /// Get the HTTP error code
    /// that should be sent to the client.
    pub fn code(&self) -> u16 {
        match self {
            Self::MissingParameter => 400,
            Self::Unauthorized => 401,
            Self::ContentTooLarge(_) => 413,
            _ => 500,
        }
    }
}

impl From<crate::controller::Error> for Error {
    fn from(error: crate::controller::Error) -> Error {
        Error::Controller(error)
    }
}

impl From<time::error::ComponentRange> for Error {
    fn from(error: time::error::ComponentRange) -> Error {
        Error::Time(error)
    }
}
