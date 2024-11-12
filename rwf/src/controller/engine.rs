//! A collection of controllers which can be mapped to
//! an arbitrary base path.
//!
//! Currently a work in progress. The closest analogy in other frameworks
//! are [Rails engines](https://guides.rubyonrails.org/engines.html).
use crate::http::{Handler, Path, Request, Response, Router};

use super::{AuthHandler, Controller, Error};

/// A collection of controllers mounted on a route.
#[derive(Default)]
pub struct Engine {
    router: Router,
    mount: Path,
    auth: Option<AuthHandler>,
}

impl Engine {
    /// Create new engine for the given routes.
    pub fn new(handlers: Vec<Handler>) -> Self {
        Self {
            router: Router::new(handlers).unwrap(),
            mount: Path::parse("/").unwrap(),
            auth: None,
        }
    }

    /// Move the engine to this mount point.
    pub fn remount(mut self, mount: &Path) -> Self {
        self.mount = mount.clone();
        self
    }

    /// Set authentication on the engine.
    pub fn auth(mut self, auth: AuthHandler) -> Self {
        self.auth = Some(auth);
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
        // Handle authentication.
        if let Some(ref auth) = self.auth {
            let auth = auth.auth();
            if !auth.authorize(request).await? {
                return auth.denied(request).await;
            }
        }

        let path = request.path().pop_base(&self.mount);
        let handler = self.router.find(&path);

        if let Some(handler) = handler {
            handler.handle(request).await
        } else {
            Ok(Response::not_found())
        }
    }
}
