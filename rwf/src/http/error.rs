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

    #[error("parameter is missing")]
    MissingParameter,

    #[error("timeout exceeded")]
    Timeout(#[from] tokio::time::error::Elapsed),

    #[error("database error: {0}")]
    Orm(#[from] crate::model::Error),

    #[error("forbidden")]
    Forbidden,
}

impl Error {
    pub fn code(&self) -> u16 {
        match self {
            Self::MissingParameter => 400,
            Self::Forbidden => 403,
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
