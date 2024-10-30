use crate::http::{Handler, Path, Request, Response, Router};

use super::{Controller, Error};

#[derive(Default)]
pub struct Engine {
    router: Router,
    mount: Path,
}

impl Engine {
    /// Create new engine for the given routes.
    pub fn new(handlers: Vec<Handler>) -> Self {
        Self {
            router: Router::new(handlers).unwrap(),
            mount: Path::parse("/").unwrap(),
        }
    }

    /// Move the engine to this mount point.
    pub fn remount(mut self, mount: &Path) -> Self {
        self.mount = mount.clone();
        self
    }

    /// Get the engine mount point.
    pub fn mount(&self) -> &Path {
        &self.mount
    }
}

#[crate::async_trait]
impl Controller for Engine {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let path = request.path().pop_base(&self.mount);
        let handler = self.router.find(&path);

        if let Some(handler) = handler {
            handler.handle(request).await
        } else {
            Ok(Response::not_found())
        }
    }
}
