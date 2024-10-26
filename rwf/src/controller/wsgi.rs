use super::{Controller, Error};
use crate::http::{wsgi::WsgiRequest, Request, Response};

use async_trait::async_trait;
use pyo3::prelude::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use tokio::sync::oneshot::channel;
use tokio::time::{timeout, Duration};

pub struct WsgiController {
    path: &'static str,
    timeout: Duration,
    pool: ThreadPool,
}

impl WsgiController {
    pub fn new(path: &'static str) -> Self {
        WsgiController {
            path,
            timeout: Duration::from_secs(60),
            pool: ThreadPoolBuilder::new().num_threads(2).build().unwrap(),
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn max_threads(mut self, threads: usize) -> Self {
        self.pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap();
        self
    }
}

#[async_trait]
impl Controller for WsgiController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let request = WsgiRequest::from_request(request)?;
        let path = self.path;
        let (tx, rx) = channel();

        self.pool.spawn(move || {
            let response = Python::with_gil(|py| {
                // Import is cached.
                let module = PyModule::import_bound(py, path).unwrap();
                let application: Py<PyAny> = module.getattr("application").unwrap().into();
                request.send(&application)
            })
            .unwrap();

            tx.send(response).unwrap();
        });

        // TODO: spawn blocking tasks cannot be aborted.
        // This only aborts waiting for the result to be returned.
        match timeout(self.timeout, rx).await? {
            Ok(response) => Ok(response.to_response()?),
            Err(e) => Ok(Response::internal_error(e)),
        }
    }
}
