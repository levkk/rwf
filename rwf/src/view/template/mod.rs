pub mod context;
pub mod error;
pub mod language;
pub mod lexer;

pub use context::Context;
pub use error::Error;
pub use lexer::{Lexer, ToValue, Token, TokenWithContext, Tokenize, Value};

use crate::http::Response;
use crate::view::Templates;

use language::Program;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::read_to_string;
use tokio::runtime::Handle;
use tokio::task;

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

    pub fn from_str(template: &str) -> Result<Self, Error> {
        Ok(Template {
            program: Program::from_str(template)?,
            path: PathBuf::from("/dev/null"),
        })
    }

    pub fn render(&self, context: impl TryInto<Context, Error = Error>) -> Result<String, Error> {
        let context: Context = context.try_into()?;

        self.program.evaluate(&context)
    }

    pub fn render_default(&self) -> Result<String, Error> {
        self.program.evaluate(&Context::default())
    }

    pub async fn cached(path: impl AsRef<Path> + Copy) -> Result<Arc<Self>, Error> {
        Templates::cache().await.get(path).await
    }

    pub async fn load(path: impl AsRef<Path> + Copy) -> Result<Arc<Self>, Error> {
        Templates::cache().await.get(path).await
    }

    pub fn load_sync(path: impl AsRef<Path> + Copy) -> Result<Arc<Self>, Error> {
        // TODO: in prod, we expect templates to be cached
        // so this should return immediately without blocking the runtime.
        // Either way, this is super ugly, and we should refactor to maybe make
        // templates async.
        task::block_in_place(move || {
            let runtime = Handle::current();
            let template = runtime.block_on(async { Template::load(path).await });

            template
        })
    }

    pub async fn cached_static(path: impl AsRef<Path> + Copy) -> Result<Response, Error> {
        match Self::cached(path).await {
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
