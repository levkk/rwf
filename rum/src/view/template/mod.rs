pub mod context;
pub mod error;
pub mod language;
pub mod lexer;

pub use context::Context;
pub use error::Error;
pub use lexer::{Lexer, ToValue, Token, TokenWithContext, Tokenize, Value};

use language::Program;

use std::path::{Path, PathBuf};
use tokio::fs::read_to_string;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Template {
    program: Program,
    path: PathBuf,
}

impl Template {
    pub async fn new(path: impl AsRef<Path> + std::marker::Copy) -> Result<Self, Error> {
        let text = match read_to_string(path).await {
            Ok(text) => text,
            Err(_) => return Err(Error::TemplateDoesNotExist(path.as_ref().to_owned())),
        };

        Ok(Template {
            program: Program::from_str(&text)?,
            path: path.as_ref().to_owned(),
        })
    }

    pub fn render(&self, context: &Context) -> Result<String, Error> {
        self.program.evaluate(context)
    }
}
