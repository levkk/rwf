use super::TokenWithContext;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("syntax error")]
    Syntax(TokenWithContext),

    #[error("eof")]
    Eof,

    #[error("undefined variable: {0}")]
    UndefinedVariable(String),
}
