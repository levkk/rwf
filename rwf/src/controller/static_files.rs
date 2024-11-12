//! Serve static files out of a folder.
//!
//! The static folder can be anywhere the application has read access to (relative or absolute path). By default, when created using [`StaticFiles::serve`] method,
//! the URL prefix and the folder are the same, e.g. `static` will serve files out of `$PWD/static` directory with the URL prefix `/static`.
//!
//! To change this behavior, create the controller with [`StaticFiles::serve`] and then call [`StaticFiles::prefix`] to set the URL prefix
//! to whatever you want.
use super::{Controller, Error};
use crate::http::{Handler, Request, Response};
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tokio::fs::File;
use tracing::debug;

/// Static files controller.
pub struct StaticFiles {
    prefix: PathBuf,
    root: PathBuf,
}

impl StaticFiles {
    /// Create a static files controller serving this path.
    ///
    /// The path can be relative, or absolute.
    pub fn serve(path: &str) -> std::io::Result<Handler> {
        let root_path = Path::new(path);
        let root = if root_path.is_absolute() {
            root_path.to_owned()
        } else {
            let cwd = std::env::current_dir()?;
            cwd.join(root_path).to_owned()
        };

        let statics = Self {
            prefix: PathBuf::from("/").join(path),
            root,
        };

        Ok(Handler::wildcard(path, statics))
    }

    /// Set the prefix used in URLs.
    ///
    /// For example, if the prefix `static` is set,
    /// all URLs to static file should start with `/static`. They
    /// will be rewritten internally to find the right file in the static
    /// folder.
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = PathBuf::from(prefix);
        self
    }
}

#[async_trait]
impl Controller for StaticFiles {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let path = request.path().to_std();

        // Remove the prefix from the request path.
        let path_components = path.components();
        let mut prefix_components = self.prefix.components();

        let path = path_components
            .filter(|path_component| {
                if let Some(prefix_component) = prefix_components.next() {
                    *path_component != prefix_component
                } else {
                    true
                }
            })
            .collect::<PathBuf>();

        // Replace the prefix with the root.
        let path = PathBuf::from(self.root.join(path));

        debug!("{} -> {}", request.path().path(), path.display());

        // Resolve all symlinks.
        let path = match tokio::fs::canonicalize(&path).await {
            Ok(path) => path,
            Err(_) => {
                return Ok(Response::not_found());
            }
        };

        // Protect against .. and symlinks going out of the root folder.
        if !path.starts_with(&self.root) {
            return Ok(Response::not_found());
        }

        match File::open(&path).await {
            Ok(file) => {
                let metadata = match file.metadata().await {
                    Ok(metadata) => metadata,
                    Err(err) => return Ok(Response::internal_error(err)),
                };

                if !metadata.is_file() {
                    return Ok(Response::not_found());
                }

                let response = Response::new();

                Ok(response.body((path, file, metadata)))
            }
            Err(_) => return Ok(Response::not_found()),
        }
    }
}
