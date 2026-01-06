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
use crate::{
    http::{Body, Handler, Request, Response},
    model::{get_connection, FromRow, Model},
    prelude::ToConnectionRequest,
};
use async_trait::async_trait;
use base64::Engine;
use sha1::{Digest, Sha1};
use std::{
    collections::HashMap,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};
use time::{macros::format_description, Duration, OffsetDateTime};
use tokio::fs::File;
use tracing::{debug, warn};

use crate::model::value::{ToValue, Value};

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

#[derive(Debug, Clone, crate::prelude::Serialize, crate::prelude::Deserialize)]
pub struct StaticFileMeta {
    id: Option<i64>,
    path: String,
    etag: String,
    modified: OffsetDateTime,
}

impl FromRow for StaticFileMeta {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, crate::model::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            path: row.try_get("path")?,
            etag: row.try_get("etag")?,
            modified: row.try_get("modified")?,
        })
    }
}

impl Model for StaticFileMeta {
    fn table_name() -> &'static str {
        "rwf_static_file_metas"
    }
    fn column_names() -> &'static [&'static str] {
        &["path", "etag", "modified"]
    }
    fn id(&self) -> Value {
        self.id.to_value()
    }
    fn values(&self) -> Vec<Value> {
        vec![
            self.path.to_value(),
            self.etag.to_value(),
            self.modified.to_value(),
        ]
    }
    fn foreign_key() -> &'static str {
        "static_file_meta_id"
    }
    fn primary_key() -> &'static str {
        "id"
    }
}

impl StaticFileMeta {
    fn format() -> &'static [time::format_description::BorrowedFormatItem<'static>] {
        format_description!(
            "[weekday repr:short], [day] [month] [year] [hour]:[minute]:[second] GMT"
        )
    }
    async fn check_request(
        request: &Request,
        conn: impl ToConnectionRequest<'_>,
    ) -> Option<Response> {
        let req_hash = request.header("if-none-match");
        //let req_modified = None::<&String>;
        let req_modified = request.header("if-modified-since");
        if req_hash.is_some() || req_modified.is_some() {
            let path = request.path().path().to_string();
            if let Ok(Some(meta)) = Self::filter("path", path.to_value())
                .unique_by(&["path"])
                .fetch_optional(conn)
                .await
            {
                // If-None-Match is prefered
                eprintln!("{:?}", req_hash);
                if let Some(etag_list) = req_hash {
                    for etag_opt in etag_list.split(",").map(|opt| opt.trim().replace("\"", "")) {
                        let etag = if etag_opt.starts_with("W/") {
                            etag_opt.strip_prefix("W/").unwrap()
                        } else {
                            etag_opt.as_str()
                        };
                        eprintln!("{}", etag);
                        if meta.etag.as_str().eq(etag) {
                            return Some(meta.add_header(Response::new().code(304)));
                        }
                    }
                }
                if let Some(req_modified) = req_modified {
                    if let Ok(modified) =
                        time::PrimitiveDateTime::parse(req_modified, Self::format())
                    {
                        let modified = OffsetDateTime::new_utc(modified.date(), modified.time());
                        eprintln!("{:?} - {:?}", modified, meta.modified);
                        if modified.ge(&meta.modified) {
                            Some(meta.add_header(Response::new().code(304)))
                        } else {
                            None
                        }
                    } else {
                        warn!("Invalid Modified Date in Request! '{}'", req_modified);
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    async fn load_by_path(
        path: impl ToString,
        conn: impl ToConnectionRequest<'_>,
    ) -> Result<Option<Self>, crate::model::Error> {
        Self::filter("path", path.to_string().to_value())
            .unique_by(&["path"])
            .fetch_optional(conn)
            .await
    }
    async fn add_preload(
        path: String,
        etag: String,
        conn: impl ToConnectionRequest<'_>,
    ) -> Result<Self, crate::model::Error> {
        Self::create(&[("path", path.to_value()), ("etag", etag.to_value())])
            .fetch(conn)
            .await
    }
    async fn add_new(
        path: String,
        etag: String,
        modified: OffsetDateTime,
        conn: impl ToConnectionRequest<'_>,
    ) -> Result<Self, crate::model::Error> {
        Self::create(&[
            ("path", path.to_value()),
            ("etag", etag.to_value()),
            ("modified", modified.to_value()),
        ])
        .fetch(conn)
        .await
    }
    async fn update(
        mut self,
        etag: String,
        modified: OffsetDateTime,
        conn: impl ToConnectionRequest<'_>,
    ) -> Result<Self, crate::model::Error> {
        if self.etag.ne(&etag) {
            self.etag = etag;
            self.modified = modified;
            self.save().fetch(conn).await
        } else {
            Ok(self)
        }
    }
    fn add_header(&self, response: Response) -> Response {
        response
            .header("etag", format!(r#"W/"{}""#, &self.etag))
            .header(
                "last-modified",
                self.modified.format(Self::format()).unwrap(),
            )
    }
}

fn default_etag_generator(data: &[u8]) -> String {
    let hash = Sha1::digest(data);
    base64::engine::general_purpose::STANDARD.encode(hash.as_slice())
}
/// Static files controller.
pub struct StaticFiles {
    prefix: PathBuf,
    root: PathBuf,
    preloads: HashMap<PathBuf, Body>,
    cache_control: CacheControl,
    etag_generator: fn(&[u8]) -> String,
    initialized: std::sync::atomic::AtomicBool,
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
            etag_generator: default_etag_generator,
            initialized: std::sync::atomic::AtomicBool::new(false),
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

    /// Sets the callback to create ETags
    pub fn etag_builder(mut self, etag_builder: fn(&[u8]) -> String) -> Self {
        self.etag_generator = etag_builder;
        self
    }

    pub fn generate_etag(&self, data: &[u8]) -> String {
        (self.etag_generator)(data)
    }

    pub fn handler(self) -> Handler {
        Handler::wildcard(self.prefix.display().to_string().as_str(), self)
    }
    pub async fn initialize(&self) -> Result<(), Error> {
        if !self.initialized.load(std::sync::atomic::Ordering::Acquire) {
            let mut conn = get_connection().await?;
            for (path, body) in self.preloads.iter() {
                match body {
                    Body::FileInclude { bytes, .. } => {
                        let path = path.as_path().as_os_str().to_string_lossy().to_string();
                        let etag = self.generate_etag(bytes.as_slice());
                        let _meta = match StaticFileMeta::load_by_path(&path, &mut conn).await? {
                            Some(obj) => {
                                obj.update(etag, OffsetDateTime::now_utc(), &mut conn)
                                    .await?
                            }
                            None => StaticFileMeta::add_preload(path, etag, &mut conn).await?,
                        };
                    }
                    _ => continue,
                }
            }
            Ok(self
                .initialized
                .store(true, std::sync::atomic::Ordering::Release))
        } else {
            Ok(())
        }
    }
    async fn file_etag(&self, path: impl AsRef<Path>) -> String {
        let data = tokio::fs::read(&path)
            .await
            .expect("File exists checked before");
        self.generate_etag(data.as_slice())
    }
}

#[async_trait]
impl Controller for StaticFiles {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        self.initialize().await?;
        let path = request.path().to_std();
        let mut conn = get_connection().await?;

        if let Some(body) = self.preloads.get(&path) {
            return if let Some(response) = StaticFileMeta::check_request(request, &mut conn).await {
                Ok(response.header("cache-control", self.cache_control.to_string()))
            } else {
                let meta = StaticFileMeta::load_by_path(request.path().path(), &mut conn)
                    .await?
                    .expect("Initialized");
                Ok(meta
                    .add_header(
                        Response::new().header("cache-control", self.cache_control.to_string()),
                    )
                    .body(body.clone()))
            };
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

                let modified = OffsetDateTime::from_unix_timestamp(metadata.mtime())
                    .expect("Unix Time Stamps should be valid");
                let meta =
                    match StaticFileMeta::load_by_path(request.path().path(), &mut conn).await? {
                        Some(meta) => {
                            if meta.modified.ne(&modified) {
                                meta.update(self.file_etag(&path).await, modified, &mut conn)
                                    .await?
                            } else {
                                if let Some(response) =
                                    StaticFileMeta::check_request(request, &mut conn).await
                                {
                                    return Ok(response
                                        .header("cache-control", self.cache_control.to_string()));
                                } else {
                                    meta
                                }
                            }
                        }
                        None => {
                            StaticFileMeta::add_new(
                                request.path().path().to_string(),
                                self.file_etag(&path).await,
                                modified,
                                &mut conn,
                            )
                            .await?
                        }
                    };

                let response = meta.add_header(
                    Response::new().header("cache-control", self.cache_control.to_string()),
                );

                Ok(response.body((path, file, metadata)))
            }
            Err(_) => Ok(Response::not_found()),
        }
    }
}
