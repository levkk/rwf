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
}

impl From<crate::controller::Error> for Error {
    fn from(error: crate::controller::Error) -> Error {
        Error::Controller(error)
    }
}
