use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("malformed query")]
    MalformedQuery,

    #[error("http error: {0}")]
    Http(http::Error),

    #[error("json error: {0}")]
    Json(serde_json::Error),
}

impl From<http::Error> for Error {
    fn from(error: http::Error) -> Self {
        Self::Http(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
