//! Implementation of the Rwf templating language.
//!
//! Templates are effectively
//! a translation of predefined functions and operations into equivalent Rust code.
//! Coupled with Rust memory management, this makes this template engine pretty fast.
//!
//! The interpreter has a lexer, parser, and an executor. For a language usage examples,
//! see [documentation](https://levkk.github.io/rwf/).
pub mod context;
pub mod error;
pub mod language;
pub mod lexer;

pub use context::Context;
pub use error::Error;
pub use lexer::{Lexer, ToTemplateValue, Token, TokenWithContext, Tokenize, Value};

use crate::http::Response;
use crate::view::Templates;

use language::Program;

use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Rwf template.
///
/// Contains the executable AST.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Template {
    program: Program,
    path: Option<PathBuf>,
}

impl Template {
    /// Read and compile a template from disk.
    pub fn new(path: impl AsRef<Path> + std::marker::Copy) -> Result<Self, Error> {
        let text = match read_to_string(path) {
            Ok(text) => text,
            Err(_) => return Err(Error::TemplateDoesNotExist(path.as_ref().to_owned())),
        };

        Ok(Template {
            program: Program::from_str(&text)?,
            path: Some(path.as_ref().to_owned()),
        })
    }

    /// Read and compile a template from a string.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::view::template::*;
    /// let template = Template::from_str("<%= 1 + 5 %>").unwrap();
    /// let result = template.render_default().unwrap();
    ///
    /// assert_eq!(result, "6");
    /// ```
    pub fn from_str(template: &str) -> Result<Self, Error> {
        Ok(Template {
            program: Program::from_str(template)?,
            path: None,
        })
    }

    /// Execute a template, provided with the context, and produce a rendering. The rendering
    /// is a string.
    pub fn render(&self, context: impl TryInto<Context, Error = Error>) -> Result<String, Error> {
        let context: Context = context.try_into()?;

        match self.program.evaluate(&context) {
            Ok(result) => Ok(result),
            Err(err) => {
                if let Some(path) = &self.path {
                    Err(err.pretty_from_path(path))
                } else {
                    Err(err)
                }
            }
        }
    }

    /// [`Self::render`] with an empty context. Used for templates that don't use any variables, or only
    /// have globally defined variables.
    pub fn render_default(&self) -> Result<String, Error> {
        self.render(&Context::default())
    }

    /// Fetch the template from cache. If the template is not in cache, load it
    /// from disk and store it in the cache for future use.
    pub fn cached(path: impl AsRef<Path> + Copy) -> Result<Arc<Self>, Error> {
        match Templates::cache().get(path) {
            Ok(template) => Ok(template),
            Err(err) => Err(err.pretty_from_path(path)),
        }
    }

    /// Load the template from disk and store it in the cache for future use. Alias for [`Self::cached`].
    pub fn load(path: impl AsRef<Path> + Copy) -> Result<Arc<Self>, Error> {
        Self::cached(path)
    }

    /// Set global default values for variables. If the variable isn't defined
    /// in a template context, and a default exists, the default value will be used instead.
    pub fn defaults(context: Context) {
        Context::defaults(context);
    }

    /// Render a static template (without variables). If the template doesn't exist
    /// and combined with the `?` operator,
    /// automatically return `500 - Internal Server Error`.
    ///
    /// Useful inside controllers.
    ///
    pub fn cached_static(path: impl AsRef<Path> + Copy) -> Result<Response, Error> {
        match Self::cached(path) {
            Ok(template) => Ok(template.try_into()?),
            Err(err) => Ok(Response::internal_error(err)),
        }
    }
}

impl TryFrom<&Template> for Response {
    type Error = Error;

    fn try_from(template: &Template) -> Result<Response, Self::Error> {
        let text = template.render_default()?;
        Ok(Response::new().html(text))
    }
}

impl TryFrom<Arc<Template>> for Response {
    type Error = Error;

    fn try_from(template: Arc<Template>) -> Result<Response, Self::Error> {
        use std::ops::Deref;
        template.deref().try_into()
    }
}
