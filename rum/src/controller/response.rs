use http_body_util::Full;
use hyper::body::Bytes;
use hyper::StatusCode;

use serde::Serialize;
use std::collections::HashMap;
use time::Duration;

use super::Error;

#[derive(Debug, Clone, Default)]
pub struct Response {
    body: Vec<u8>,
    headers: HashMap<String, String>,
    status: StatusCode,
}

impl Response {
    pub fn new() -> Self {
        Self::default().header("Server", "rum")
    }

    pub fn header(mut self, name: impl ToString, value: impl ToString) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    pub fn body(mut self, body: &[u8]) -> Self {
        self.body = body.to_vec();
        self.header("Content-Length", body.len())
    }

    pub fn json(value: impl Serialize) -> Result<Self, Error> {
        let mut response = Response::new()
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&value)?.as_bytes());
        Ok(response)
    }

    pub fn html(value: impl ToString) -> Self {
        Response::new()
            .header("Content-Type", "text/html; charset=UTF-8")
            .body(value.to_string().as_bytes())
    }

    pub fn text(value: impl ToString) -> Self {
        Response::new()
            .header("Content-Type", "text/plain")
            .body(value.to_string().as_bytes())
    }

    pub fn no_cache(self) -> Self {
        self.header("Cache-Control", "no-store")
    }

    pub fn cache(self, duration: Duration) -> Self {
        self.header(
            "Cache-Control",
            format!("public, max-age={:.0}", duration.as_seconds_f64()),
        )
    }
}

impl TryFrom<Response> for hyper::Response<Vec<u8>> {
    type Error = Error;

    fn try_from(response: Response) -> Result<hyper::Response<Vec<u8>>, Self::Error> {
        let mut builder = hyper::Response::builder().status(response.status);
        for (key, value) in response.headers {
            builder = builder.header(key, value);
        }

        Ok(builder.body(response.body)?)
    }
}

impl TryFrom<Response> for hyper::Response<Full<Bytes>> {
    type Error = Error;

    fn try_from(response: Response) -> Result<hyper::Response<Full<Bytes>>, Self::Error> {
        let mut builder = hyper::Response::builder().status(response.status);
        for (key, value) in response.headers {
            builder = builder.header(key, value);
        }

        Ok(builder.body(response.body.into())?)
    }
}
