use super::{Controller, Error};
use crate::http::{Body, Request, Response};
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tokio::fs::{canonicalize, File};

pub struct StaticFiles {
    prefix: PathBuf,
    root: PathBuf,
}

#[async_trait]
impl Controller for StaticFiles {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let path = request.path().to_std();

        if !path.starts_with(&self.prefix) {
            return Ok(Response::not_found());
        }

        let path = path.display().to_string().replace(
            self.prefix.display().to_string().as_str(),
            self.root.display().to_string().as_str(),
        );
        let path = PathBuf::from(path);
        let path = match tokio::fs::canonicalize(path).await {
            Ok(path) => path,
            Err(_) => return Ok(Response::not_found()),
        };

        // Protect against .. and symlinks going out of the root folder.
        if !path.starts_with(&self.root) {
            return Ok(Response::not_found());
        }

        match File::open(&path).await {
            Ok(file) => {
                let response = Response::from_request(request)?;
                let metadata = match file.metadata().await {
                    Ok(metadata) => metadata,
                    Err(err) => return Ok(Response::internal_error(err)),
                };

                Ok(response.body((path, file, metadata)))
            }
            Err(_) => return Ok(Response::not_found()),
        }
    }
}
