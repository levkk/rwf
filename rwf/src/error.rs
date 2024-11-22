//! Global error type.
use thiserror::Error;

/// An error returned by any Rwf module.
#[derive(Error, Debug)]
pub enum Error {
    /// IO error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON (de)serialization error.
    #[error("json")]
    Json(#[from] serde_json::Error),

    /// Error returned by a controller.
    #[error("{0}")]
    Controller(#[from] crate::controller::Error),

    /// Error returned by the HTTP server.
    #[error("{0}")]
    Http(#[from] crate::http::Error),

    /// Error returned by crypto.
    #[error("{0}")]
    Crypto(#[from] crate::crypto::Error),

    /// Error returned by the template engine.
    #[error("{0}")]
    View(#[from] crate::view::Error),

    /// Error returned by the background jobs queue.
    #[error("{0}")]
    Job(#[from] crate::job::Error),

    /// Error returned by comms.
    #[error("{0}")]
    Comms(#[from] crate::comms::Error),

    /// Utf-8 decoding error.
    #[error("{0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Regex error.
    #[error("{0}")]
    Regex(#[from] regex::Error),

    /// Time error.
    #[error("{0}")]
    Time(time::error::ComponentRange),

    /// Format error.
    #[error("fmt error: {0}")]
    FmtError(#[from] std::fmt::Error),

    /// Any error that implements `std::error::Error`.
    #[error("{0}")]
    Error(#[from] Box<dyn std::error::Error + Sync + Send>),
}
