use super::{Token, TokenWithContext};
use thiserror::Error;

use std::path::{Path, PathBuf};

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

    #[error("serialization error")]
    SerializationError,

    #[error("time format error: {0}")]
    TimeFormatError(#[from] time::error::Format),

    #[error("{0}")]
    Pretty(String),
}

impl Error {
    pub fn pretty(self, source: &str, path: Option<impl AsRef<Path> + Copy>) -> Self {
        let token = match self {
            Error::Syntax(ref token) => token,
            Error::ExpressionSyntax(ref token) => token,
            Error::WrongToken(ref token, _) => token,
            _ => return self,
        };

        let error_msg = match self {
            Error::Syntax(ref _token) => "syntax error".to_string(),
            Error::ExpressionSyntax(ref _token) => "expression syntax error".to_string(),
            Error::WrongToken(ref _token, _) => "unexpected token".to_string(),
            _ => "".to_string(),
        };

        let context = source.lines().nth(token.line() - 1); // std::fs lines start at 0
        let leading_spaces = if let Some(ref context) = context {
            context.len() - context.trim().len()
        } else {
            0
        };
        let underline = vec![' '; token.column() - token.token().len() + 1 - leading_spaces]
            .into_iter()
            .collect::<String>()
            + &format!("^ {}", error_msg);

        let line_number = format!("{} | ", token.line());
        let underline_offset = vec![' '; token.line().to_string().len()]
            .into_iter()
            .collect::<String>()
            + " | ";

        let path = if let Some(path) = path {
            format!(
                "---> {}:{}:{}\n\n",
                path.as_ref().display(),
                token.line(),
                token.column()
            )
        } else {
            "".to_string()
        };

        if let Some(context) = context {
            Error::Pretty(format!(
                "{}{}\n{}{}\n{}{}",
                path,
                underline_offset,
                line_number,
                context.trim(),
                underline_offset,
                underline
            ))
        } else {
            self
        }
    }

    pub fn pretty_from_path(self, path: impl AsRef<Path> + Copy) -> Self {
        let src = match std::fs::read_to_string(path) {
            Ok(src) => src,
            Err(_) => return self,
        };

        self.pretty(&src, Some(path))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_underline() {
        let token = TokenWithContext::new(Token::If, 1, 9);
        let error = Error::Syntax(token);
        let pretty = error.pretty(
            "<% if apples %>
    <% if oranges are blue %>
",
            None::<&str>,
        );

        assert_eq!(
            pretty.to_string(),
            "    <% if oranges are blue %>
       ^ syntax error"
        );
    }
}
