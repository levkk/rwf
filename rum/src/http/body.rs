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
            Html(html) => html.len(),
            Json(json) => json.len(),
            Text(text) => text.len(),
        }
    }

    pub fn mime_type(&self) -> &'static str {
        use Body::*;

        match self {
            File { path, .. } => {
                let extension = match path.extension() {
                    Some(extension) => extension.to_str().expect("OsStr to_str"),
                    None => "",
                };

                match extension {
                    "pdf" => "application/pdf",
                    "json" => "application/json",
                    "js" => "application/javascript",
                    "css" => "text/css",
                    "html" => "text/html",
                    "txt" => "text/plain",
                    "png" => "application/png",
                    "apng" => "application/apng",
                    "svg" => "application/svg+xml",
                    "jpg" | "jpeg" => "application/jpeg",
                    "webp" => "application/webp",
                    "xml" => "application/xml",
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
