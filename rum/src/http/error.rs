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
}

impl From<crate::controller::Error> for Error {
    fn from(error: crate::controller::Error) -> Error {
        Error::Controller(error)
    }
}