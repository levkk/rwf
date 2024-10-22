use super::{Controller, Error};
use crate::http::{wsgi::WsgiRequest, Request, Response};

use async_trait::async_trait;
use pyo3::prelude::*;
use tokio::time::{timeout, Duration};

pub struct WsgiController {
    path: &'static str,
    timeout: Duration,
}

impl WsgiController {
    pub fn new(path: &'static str) -> Self {
        WsgiController {
            path,
            timeout: Duration::from_secs(60),
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[async_trait]
impl Controller for WsgiController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let request = WsgiRequest::from_request(request)?;
        let path = self.path;

        // TODO: spawn blocking tasks cannot be aborted.
        // This only aborts waiting for the result to be returned.
        match timeout(
            self.timeout,
            tokio::task::spawn_blocking(move || {
                let response = Python::with_gil(|py| {
                    // Import is cached.
                    let module = PyModule::import_bound(py, path).unwrap();
                    let application: Py<PyAny> = module.getattr("application").unwrap().into();
                    request.send(&application)
                })
                .unwrap();

                response
            }),
        )
        .await?
        {
            Ok(response) => Ok(response.to_response()?),
            Err(e) => Ok(Response::internal_error(e)),
        }
    }
}
