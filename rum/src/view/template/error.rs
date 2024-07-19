use super::{Token, TokenWithContext};
use thiserror::Error;

use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum Error {
    #[error("syntax error")]
    Syntax(TokenWithContext),

    #[error("expression syntax error")]
    ExpressionSyntax(TokenWithContext),

    #[error("expected {0}, got {0}")]
    WrongToken(TokenWithContext, Token),

    #[error("eof: {0}")]
    Eof(&'static str),

    #[error("undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("unknown method: {0}")]
    UnknownMethod(String),

    #[error("template does not exist: {0}")]
    TemplateDoesNotExist(PathBuf),
}
