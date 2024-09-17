use serde::Serialize;
use std::collections::HashMap;
use std::marker::Unpin;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::{head::Version, Error, Headers};

#[derive(Debug)]
pub enum Status {
    NotFound,
    InternalServerError,
    MethodNotAllowed,
    Code(u16),
}

impl Status {
    pub fn code(&self) -> u16 {
        use Status::*;

        match self {
            NotFound => 404,
            InternalServerError => 500,
            MethodNotAllowed => 405,
            Code(code) => *code,
        }
    }
}

impl From<u16> for Status {
    fn from(code: u16) -> Status {
        use Status::*;

        match code {
            404 => NotFound,
            500 => InternalServerError,
            405 => MethodNotAllowed,
            code => Code(code),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    code: u16,
    headers: Headers,
    version: Version,
    body: Vec<u8>,
}

impl Response {
    pub fn new() -> Self {
        Self {
            code: 200,
            headers: Headers::from(HashMap::from([
                ("content-type".to_string(), "text/plain".to_string()),
                ("server".to_string(), "rum".to_string()),
                ("connection".to_string(), "keep-alive".to_string()),
            ])),
            body: Vec::new(),
            version: Version::Http1,
        }
    }

    fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self.headers
            .insert("content-length".to_string(), self.body.len().to_string());
        self
    }

    pub fn status(&self) -> Status {
        self.code.into()
    }

    pub fn code(mut self, code: u16) -> Self {
        self.code = code;
        self
    }

    pub fn json(body: impl Serialize) -> Result<Self, Error> {
        let body = serde_json::to_vec(&body)?;
        Ok(Self::new()
            .header("content-type", "application/json")
            .body(body))
    }

    pub fn html(body: impl ToString) -> Self {
        Self::new()
            .header("content-type", "text/html")
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
        let mut response = format!("{} {}\r\n", self.version, self.code)
            .as_bytes()
            .to_vec();
        response.extend_from_slice(&self.headers.to_bytes());
        response.extend_from_slice(b"\r\n");
        response.extend_from_slice(&self.body);

        stream.write_all(&response).await
    }

    pub fn not_found() -> Self {
        Self::html(
            "
            <h3>
                <center>404 - Not Found</center>
            </h3>
        ",
        )
        .code(404)
    }

    pub fn method_not_allowed() -> Self {
        Self::html(
            "
            <h3>
                <center>405 - Method Not Allowed</center>
            </h3>
        ",
        )
        .code(405)
    }

    pub fn bad_request() -> Self {
        Self::html(
            "
            <h3>
                <center>400 - Bad Request</center>
            </h3>
        ",
        )
        .code(400)
    }

    pub fn not_implemented() -> Self {
        Self::html(
            "
            <h3>
                <center>501 - Not Implemented</center>
            </h3>
            ",
        )
        .code(501)
    }

    pub fn internal_error(_err: impl std::error::Error) -> Self {
        Self::html(
            "
            <h3>
                <center>500 - Internal Server Error</center>
            </h3>
            ",
        )
        .code(500)
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
