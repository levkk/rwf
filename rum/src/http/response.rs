use serde::Serialize;
use std::collections::HashMap;
use std::marker::Unpin;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::{Error, Headers};

#[derive(Debug, Clone)]
pub struct Response {
    code: u16,
    headers: Headers,
    body: Vec<u8>,
}

impl Response {
    pub fn new() -> Self {
        Self {
            code: 200,
            headers: Headers::from(HashMap::from([
                ("content-type".to_string(), "text/plain".to_string()),
                ("server".to_string(), "rum".to_string()),
            ])),
            body: Vec::new(),
        }
    }

    fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self.headers
            .insert("content-length".to_string(), self.body.len().to_string());
        self
    }

    pub fn code(mut self, code: u16) -> Self {
        self.code = code;
        self
    }

    pub fn get_code(&self) -> u16 {
        self.code
    }

    pub fn json(self, body: impl Serialize) -> Result<Self, Error> {
        let body = serde_json::to_vec(&body)?;
        Ok(self.header("content-type", "application/json").body(body))
    }

    pub fn html(self, body: impl ToString) -> Self {
        self.header("content-type", "text/html")
            .body(body.to_string().as_bytes().to_vec())
    }

    pub fn text(self, body: impl ToString) -> Result<Self, Error> {
        Ok(self
            .header("content-type", "text/plain")
            .body(body.to_string().as_bytes().to_vec()))
    }

    pub fn header(mut self, name: impl ToString, value: impl ToString) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    pub async fn send(&self, mut stream: impl AsyncWrite + Unpin) -> Result<(), std::io::Error> {
        let mut response = format!("HTTP/1.1 {}\r\n", self.code).as_bytes().to_vec();
        response.extend_from_slice(&self.headers.as_bytes());
        response.extend_from_slice(b"\r\n");
        response.extend_from_slice(&self.body);

        stream.write_all(&response).await
    }
}

pub trait ToResponse {
    fn to_response(self) -> Result<Response, Error>;
}

impl ToResponse for String {
    fn to_response(self) -> Result<Response, Error> {
        Response::new().text(self)
    }
}

impl ToResponse for Response {
    fn to_response(self) -> Result<Response, Error> {
        Ok(self)
    }
}

impl ToResponse for &str {
    fn to_response(self) -> Result<Response, Error> {
        Response::new().text(self)
    }
}
