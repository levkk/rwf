use http_body_util::BodyExt;
use hyper::header::{HeaderMap, CONTENT_TYPE};

use super::{route::Query, Error};

#[derive(Debug)]
pub struct Request {
    request: hyper::Request<hyper::body::Incoming>,
    query: Query,
}

impl Request {
    pub fn new(request: hyper::Request<hyper::body::Incoming>) -> Result<Self, Error> {
        Ok(Self {
            query: Query::try_from(request.uri().query().unwrap_or(""))?,
            request,
        })
    }

    pub fn is_json(&self) -> bool {
        self.headers()
            .get(CONTENT_TYPE)
            .map_or(false, |v| v == "application/json")
    }

    pub fn headers(&self) -> &HeaderMap {
        self.request.headers()
    }

    pub async fn body(self) -> Result<Vec<u8>, Error> {
        Ok(self
            .request
            .into_body()
            .collect()
            .await?
            .to_bytes()
            .to_vec())
    }

    pub async fn json(self) -> Result<serde_json::Value, Error> {
        Ok(serde_json::from_slice(&self.body().await?)?)
    }
}
