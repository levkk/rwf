use super::{Controller, Error};
use crate::http::{wsgi::WsgiRequest, Request, Response};

use async_trait::async_trait;
use pyo3::prelude::*;

use tokio::time::{timeout, Duration};

pub struct WsgiController {
    path: &'static str,
}

impl WsgiController {
    pub fn new(path: &'static str) -> Self {
        WsgiController { path }
    }
}

#[async_trait]
impl Controller for WsgiController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let request = WsgiRequest::from_request(request)?;
        let path = self.path;

        let response = timeout(
            Duration::from_secs(5),
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
        .await
        .unwrap()
        .unwrap();

        Ok(response.to_response()?)
    }
}
