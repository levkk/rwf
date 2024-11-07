use std::fmt::Debug;
use std::fs::Metadata;
use std::marker::Unpin;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{copy, AsyncWrite, AsyncWriteExt};

#[derive(Debug)]
pub enum Body {
    File {
        path: PathBuf,
        file: File,
        metadata: Metadata,
    },
    Html(String),
    Bytes(Vec<u8>),
    Text(String),
    Json(Vec<u8>),
}

impl Body {
    pub fn bytes(bytes: Vec<u8>) -> Self {
        Self::Bytes(bytes)
    }

    pub fn html(text: impl ToString) -> Self {
        Self::Html(text.to_string())
    }

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
        }
    }

    pub fn len(&self) -> usize {
        use Body::*;

        match self {
            File { metadata, .. } => metadata.len() as usize,
            Bytes(bytes) => bytes.len(),
            Html(html) => html.as_bytes().len(),
            Json(json) => json.len(),
            Text(text) => text.as_bytes().len(),
        }
    }

    pub fn mime_type(&self) -> &'static str {
        use Body::*;

        match self {
            File { path, .. } => {
                // Guessing the mime by the extension.
                let extension = match path.extension() {
                    Some(extension) => extension.to_str().expect("OsStr to_str"),
                    None => "",
                };

                // https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types
                match extension {
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
            Html(_) => "text/html",
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
