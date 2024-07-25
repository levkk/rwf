use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("malformed request: {0}")]
    MalformedRequest(&'static str),

    #[error("json")]
    Json(#[from] serde_json::Error),
}
