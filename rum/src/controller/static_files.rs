use super::{Controller, Error};
use crate::http::{Handler, Request, Response};
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tokio::fs::File;

pub struct StaticFiles {
    prefix: PathBuf,
    root: PathBuf,
}

impl StaticFiles {
    pub fn serve(path: &str) -> std::io::Result<Handler> {
        let cwd = std::env::current_dir()?;
        let root = cwd.join(Path::new(path));

        let statics = Self {
            prefix: PathBuf::from(path),
            root,
        };

        Ok(Handler::new(path, statics))
    }
}

#[async_trait]
impl Controller for StaticFiles {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let path = request.path().to_std();

        let path = path.display().to_string().replace(
            self.prefix.display().to_string().as_str(),
            self.root.display().to_string().as_str(),
        );

        let path = PathBuf::from(self.root.join(path));
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
