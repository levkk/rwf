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

impl Response {}
