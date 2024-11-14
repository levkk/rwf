//! Global error type.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("malformed request: {0}")]
    MalformedRequest(&'static str),

    #[error("json")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Controller(crate::controller::Error),

    #[error("{0}")]
    Crypto(#[from] crate::crypto::Error),

    #[error("{0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("{0}")]
    Regex(#[from] regex::Error),

    #[error("{0}")]
    Time(time::error::ComponentRange),

    #[error("fmt error: {0}")]
    FmtError(#[from] std::fmt::Error),

    #[error("{0}")]
    Error(#[from] Box<dyn std::error::Error + Sync + Send>),
}
