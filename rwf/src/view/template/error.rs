use super::{Token, TokenWithContext};
use thiserror::Error;

use std::path::{Path, PathBuf};

#[derive(Error, Debug)]
pub enum Error {
    #[error("syntax error")]
    Syntax(TokenWithContext),

    #[error("expression syntax error")]
    ExpressionSyntax(TokenWithContext),

    #[error("expected token \"{0}\", but have token \"{0}\" instead")]
    WrongToken(TokenWithContext, Token),

    #[error("reached end of file while performing \"{0}\", did you forget a closing tag?")]
    Eof(&'static str),

    #[error("variable \"{0}\" is not defined or in scope")]
    UndefinedVariable(String),

    #[error("method \"{0}\" is not defined")]
    UnknownMethod(String),

    #[error("template \"{0}\" does not exist")]
    TemplateDoesNotExist(PathBuf),

    #[error("serialization error")]
    SerializationError,

    #[error("failed to format a timtestamp correctly, error: \"{0}\"")]
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
            _ => {
                if let Some(path) = path {
                    let prefix = "---> ";
                    return Error::Pretty(format!(
                        "{}{}\n\n{}{}",
                        prefix,
                        path.as_ref().display(),
                        vec![' '; prefix.len()].into_iter().collect::<String>(),
                        self.to_string()
                    ));
                } else {
                    return self;
                }
            }
        };

        let error_msg = match self {
            Error::Syntax(ref _token) => "syntax error".to_string(),
            Error::ExpressionSyntax(ref _token) => "expression syntax error".to_string(),
            Error::WrongToken(ref _token, _) => "unexpected token".to_string(),
            _ => "".to_string(),
        };

        println!(
            "token {:?}, {}, {}",
            token,
            token.line(),
            token.token().len()
        );

        let context = source.lines().nth(std::cmp::max(1, token.line()) - 1); // std::fs lines start at 0
        let leading_spaces = if let Some(ref context) = context {
            context.len() - context.trim().len()
        } else {
            0
        };
        println!("leading spaces: {}", leading_spaces);
        let underline = vec![
            ' ';
            std::cmp::max(
                0,
                token.column() as i64 - token.token().len() as i64 + 1 - leading_spaces as i64
            ) as usize
        ]
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
        #[cfg(debug_assertions)]
        {
            let src = match std::fs::read_to_string(path) {
                Ok(src) => src,
                Err(_) => return self,
            };

            self.pretty(&src, Some(path))
        }

        #[cfg(not(debug_assertions))]
        {
            let _ = path;
            self
        }
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
            "  | \n1 | <% if apples %>\n  |         ^ syntax error"
        );
    }
}
