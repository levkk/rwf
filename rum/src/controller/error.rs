use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("malformed query")]
    MalformedQuery,

    #[error("http error: {0}")]
    Http(#[from] http::Error),

    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("orm error: {0}")]
    OrmError(#[from] crate::model::Error),
}
