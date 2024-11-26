//! Serve static files out of a folder.
//!
//! The static folder can be anywhere the application has read access to (relative or absolute path).
//! By default, when created using [`StaticFiles::serve`] method,
//! the URL prefix and the folder are the same, e.g. `static` will serve files out of `$PWD/static` directory
//! with the URL prefix `/static`.
//!
//! To change this behavior, create the controller with [`StaticFiles::serve`] and
//! then call [`StaticFiles::prefix`] to set the URL prefix to whatever you want.
use super::{Controller, Error};
use crate::http::{Body, Handler, Request, Response};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use time::Duration;
use tokio::fs::File;
use tracing::debug;

/// Cache control header.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CacheControl {
    NoStore,
    MaxAge(Duration),
    Private,
    NoCache,
}

impl std::fmt::Display for CacheControl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use CacheControl::*;
        let s = match self {
            NoStore => "no-store".into(),
            MaxAge(duration) => format!("max-age={}", duration.whole_seconds()),
            Private => "private".into(),
            NoCache => "no-cache".into(),
        };

        write!(f, "{}", s)
    }
}

/// Static files controller.
pub struct StaticFiles {
    prefix: PathBuf,
    root: PathBuf,
    preloads: HashMap<PathBuf, Body>,
    cache_control: CacheControl,
}

impl StaticFiles {
    /// Create a controller handler to serve static files from this path.
    pub fn serve(path: &str) -> std::io::Result<Handler> {
        let statics = Self::new(path)?;

        Ok(statics.handler())
    }

    /// Create a static files controller seriving this path. The path can be
    /// relative or absolute.
    pub fn new(path: &str) -> std::io::Result<Self> {
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
            preloads: HashMap::new(),
            cache_control: CacheControl::NoStore,
        };

        Ok(statics)
    }

    /// Serve static files with the specified `Cache-Control: max-age` attribute.
    pub fn cached(path: &str, duration: Duration) -> std::io::Result<Handler> {
        Ok(Self::new(path)?
            .cache_control(CacheControl::MaxAge(duration))
            .handler())
    }

    /// Preload a static file into memory. This allows static files to load and serve files
    /// which may not be available at runtime, e.g. by using [`include_bytes`].
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::controller::StaticFiles;
    /// StaticFiles::new("static")
    ///     .unwrap()
    ///     .preload("/style.css", b"body { background: black; }");
    /// ```
    pub fn preload(mut self, path: impl AsRef<Path> + Copy, bytes: &[u8]) -> Self {
        self.preloads.insert(
            path.as_ref().to_owned(),
            Body::file_include(&path.as_ref().to_owned(), bytes.to_vec()),
        );
        self
    }

    /// Set the `Cache-Control` header.
    pub fn cache_control(mut self, cache_control: CacheControl) -> Self {
        self.cache_control = cache_control;
        self
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

    pub fn handler(self) -> Handler {
        Handler::wildcard(self.prefix.display().to_string().as_str(), self)
    }
}

#[async_trait]
impl Controller for StaticFiles {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let path = request.path().to_std();

        if let Some(body) = self.preloads.get(&path) {
            return Ok(Response::new().body(body.clone()));
        }

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

                let response =
                    Response::new().header("cache-control", self.cache_control.to_string());

                Ok(response.body((path, file, metadata)))
            }
            Err(_) => return Ok(Response::not_found()),
        }
    }
}
