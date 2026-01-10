//! Handle sending a response body to the client.
//!
//! The body can be text, HTML, raw bytes, JSON and a static file. The `Content-Type` and `Content-Length` headers
//! are set automatically.
use std::fmt::Debug;
use std::fs::Metadata;
use std::marker::Unpin;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{copy, AsyncWrite, AsyncWriteExt};

/// Response body.
#[derive(Debug)]
pub enum Body {
    /// Static file.
    File {
        path: PathBuf,
        file: File,
        metadata: Metadata,
    },
    /// UTF-8 encoded HTML.
    Html(String),
    /// Raw bytes.
    Bytes(Vec<u8>),
    /// UTF-8 encoded text.
    Text(String),
    /// UTF-8 encoded JSON string.
    Json(Vec<u8>),
    /// A file that's already read into memory.
    FileInclude { path: PathBuf, bytes: Vec<u8> },
}

impl Clone for Body {
    /// Clone the body.
    ///
    /// # Panics
    ///
    /// Will panic if [`Body::File`] is cloned.
    fn clone(&self) -> Self {
        use Body::*;
        match self {
            FileInclude { path, bytes } => Body::FileInclude {
                path: path.clone(),
                bytes: bytes.clone(),
            },
            Html(html) => Html(html.clone()),
            Text(text) => Text(text.clone()),
            Json(json) => Json(json.clone()),
            Bytes(bytes) => Bytes(bytes.clone()),
            File { .. } => {
                panic!("file body cannot be cloned, it contains an open file descriptor")
            }
        }
    }
}

impl Body {
    /// Create new body from raw bytes.
    pub fn bytes(bytes: Vec<u8>) -> Self {
        Self::Bytes(bytes)
    }

    /// Create new body from a string assumed to be HTML.
    pub fn html(text: impl ToString) -> Self {
        Self::Html(text.to_string())
    }

    /// Create a new static file that's already loaded into memory.
    pub fn file_include(path: &PathBuf, bytes: Vec<u8>) -> Self {
        Self::FileInclude {
            path: path.to_owned(),
            bytes,
        }
    }

    /// Send the body to the stream. If the body is a file,
    /// it will be sent efficiently using [`tokio::io::copy`].
    /// The stream is not flushed, so if call `stream.flush().await`
    /// to make sure the data reaches the client.
    pub async fn send(
        &mut self,
        mut stream: impl AsyncWrite + Unpin,
    ) -> Result<(), std::io::Error> {
        use Body::*;

        match self {
            File { file, .. } => {
                copy(file, &mut stream).await?;
                Ok(())
            }
            Bytes(bytes) => Ok(stream.write_all(bytes).await?),
            Text(text) => Ok(stream.write_all(text.as_bytes()).await?),
            Html(html) => Ok(stream.write_all(html.as_bytes()).await?),
            Json(json) => Ok(stream.write_all(json.as_slice()).await?),
            FileInclude { bytes, .. } => Ok(stream.write_all(bytes).await?),
        }
    }

    /// Get the body size. Used in the `Content-Length` header.
    pub fn len(&self) -> usize {
        use Body::*;

        match self {
            File { metadata, .. } => metadata.len() as usize,
            Bytes(bytes) => bytes.len(),
            Html(html) => html.len(),
            Json(json) => json.len(),
            Text(text) => text.len(),
            FileInclude { bytes, .. } => bytes.len(),
        }
    }

    /// Get the body's MIME type. This determines the value of the `Content-Type` header.
    ///
    /// If the body is a file, this will guess the mime type from the file extension.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::http::Body;
    /// let body = Body::html("<h1>Hello from Rwf!</h1>");
    /// assert_eq!(body.mime_type(), "text/html; charset=utf-8");
    /// ```
    pub fn mime_type(&self) -> &'static str {
        use Body::*;

        match self {
            File { path, .. } | FileInclude { path, .. } => {
                // Guessing the mime by the extension.
                let extension = match path.extension() {
                    Some(extension) => extension.to_str().expect("OsStr to_str"),
                    None => "",
                }
                .to_lowercase();

                // https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types
                match extension.as_str() {
                    "aac" => "audio/aac",
                    "abw" => "application/x-abiword",
                    "arc" => "application/x-freearc",
                    "avif" => "image/avif",
                    "avi" => "video/x-msvideo",
                    "azw" => "application/vnd.amazon.ebook",
                    "bin" => "application/octet-stream",
                    "bmp" => "image/bmp",
                    "bz" => "application/x-bzip",
                    "bz2" => "application/x-bzip2",
                    "cda" => "application/x-cdf",
                    "csh" => "application/x-csh",
                    "css" => "text/css",
                    "csv" => "text/csv",
                    "doc" => "application/msword",
                    "docx" => {
                        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                    }
                    "eot" => "application/vnd.ms-fontobject",
                    "epub" => "application/epub+zip",
                    "gz" => "application/gzip",
                    "gif" => "image/gif",
                    "htm" => "text/html",
                    "html" => "text/html",
                    "ico" => "image/vnd.microsoft.icon",
                    "ics" => "text/calendar",
                    "jar" => "application/java-archive",
                    "jpeg" => "image/jpeg",
                    "jpg" => "image/jpeg",
                    "js" => "text/javascript",
                    "json" => "application/json",
                    "jsonld" => "application/ld+json",
                    "mid" => "audio/midi",
                    "midi" => "audio/midi",
                    "mjs" => "text/javascript",
                    "mp3" => "audio/mpeg",
                    "mp4" => "video/mp4",
                    "mpeg" => "video/mpeg",
                    "mpkg" => "application/vnd.apple.installer+xml",
                    "odp" => "application/vnd.oasis.opendocument.presentation",
                    "ods" => "application/vnd.oasis.opendocument.spreadsheet",
                    "odt" => "application/vnd.oasis.opendocument.text",
                    "oga" => "audio/ogg",
                    "ogv" => "video/ogg",
                    "ogx" => "application/ogg",
                    "opus" => "audio/opus",
                    "otf" => "font/otf",
                    "png" => "image/png",
                    "pdf" => "application/pdf",
                    "php" => "application/x-httpd-php",
                    "ppt" => "application/vnd.ms-powerpoint",
                    "pptx" => {
                        "application/vnd.openxmlformats-officedocument.presentationml.presentation"
                    }
                    "rar" => "application/vnd.rar",
                    "rtf" => "application/rtf",
                    "sh" => "application/x-sh",
                    "svg" => "image/svg+xml",
                    "tar" => "application/x-tar",
                    "tif" => "image/tiff",
                    "tiff" => "image/tiff",
                    "ts" => "video/mp2t",
                    "ttf" => "font/ttf",
                    "txt" => "text/plain",
                    "vsd" => "application/vnd.visio",
                    "wav" => "audio/wav",
                    "weba" => "audio/webm",
                    "webm" => "video/webm",
                    "webp" => "image/webp",
                    "woff" => "font/woff",
                    "woff2" => "font/woff2",
                    "xhtml" => "application/xhtml+xml",
                    "xls" => "application/vnd.ms-excel",
                    "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    "xml" => "application/xml",
                    "xul" => "application/vnd.mozilla.xul+xml",
                    "zip" => "application/zip",
                    "3gp" => "video/3gpp",
                    "3g2" => "video/3gpp2",
                    "7z" => "application/x-7z-compressed",
                    _ => "application/octet-stream",
                }
            }
            Text(_) => "text/plain",
            Html(_) => "text/html; charset=utf-8",
            Json(_) => "application/json",
            Bytes(_) => "application/octet-stream",
        }
    }
}

impl From<Vec<u8>> for Body {
    fn from(body: Vec<u8>) -> Self {
        Self::Bytes(body)
    }
}

impl From<&[u8]> for Body {
    fn from(body: &[u8]) -> Self {
        Self::Bytes(body.to_vec())
    }
}

impl From<(PathBuf, File, Metadata)> for Body {
    fn from(file: (PathBuf, File, Metadata)) -> Self {
        Self::File {
            path: file.0,
            file: file.1,
            metadata: file.2,
        }
    }
}

impl TryFrom<serde_json::Value> for Body {
    type Error = serde_json::Error;

    fn try_from(json: serde_json::Value) -> Result<Self, Self::Error> {
        Ok(Self::Json(serde_json::to_vec(&json)?))
    }
}
