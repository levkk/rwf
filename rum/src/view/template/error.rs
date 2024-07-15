use super::TokenWithLine;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("syntax error")]
    Syntax(TokenWithLine),

    #[error("eof")]
    Eof,

    #[error("undefined variable: {0}")]
    UndefinedVariable(String),
}
