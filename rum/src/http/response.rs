//! HTTP response.

use serde::Serialize;
use std::collections::HashMap;
use std::marker::Unpin;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::{head::Version, Error, Headers};

/// Response status, e.g. 404, 200, etc.
#[derive(Debug)]
pub enum Status {
    NotFound,
    InternalServerError,
    MethodNotAllowed,
    Ok,
    Created,
    Code(u16),
}

impl Status {
    pub fn code(&self) -> u16 {
        use Status::*;

        match self {
            NotFound => 404,
            InternalServerError => 500,
            MethodNotAllowed => 405,
            Ok => 200,
            Created => 201,
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
            200 => Ok,
            201 => Created,
            code => Code(code),
        }
    }
}

/// HTTP response.
#[derive(Debug, Clone)]
pub struct Response {
    code: u16,
    headers: Headers,
    version: Version,
    body: Vec<u8>,
}

impl Response {
    /// Create empty response.
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

    pub fn as_slice(&self) -> &[u8] {
        &self.body
    }

    /// Response status, e.g. 200 OK.
    pub fn status(&self) -> Status {
        self.code.into()
    }

    /// Set response code.
    ///
    /// # Example
    ///
    /// ```
    /// use rum::http::Response;
    ///
    /// let response = Response::text("OK").code(200);
    /// ```
    pub fn code(mut self, code: u16) -> Self {
        self.code = code;
        self
    }

    /// Create a response with a JSON body serialized from a Rust type.
    ///
    /// # Example
    ///
    /// ```
    /// use rum::http::Response;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Body {
    ///     value: String,
    /// }
    ///
    /// let response = Response::json(Body { value: "hello world".to_string() })
    ///    .unwrap()
    ///    .code(200);
    /// ```
    pub fn json(body: impl Serialize) -> Result<Self, Error> {
        let body = serde_json::to_vec(&body)?;
        Ok(Self::new()
            .header("content-type", "application/json")
            .body(body))
    }

    /// Create a response with an HTML body.
    ///
    /// # Example
    ///
    /// ```
    /// use rum::http::Response;
    ///
    /// let response = Response::html("<h1>Hello world</h1>");
    /// ```
    pub fn html(body: impl ToString) -> Self {
        Self::new()
            .header("content-type", "text/html")
            .body(body.to_string().as_bytes().to_vec())
    }

    /// Create a response with a plain text body.
    ///
    /// # Example
    ///
    /// ```
    /// use rum::http::Response;
    ///
    /// let response = Response::text("Hello world");
    /// ```
    pub fn text(body: impl ToString) -> Self {
        Self::new()
            .header("content-type", "text/plain")
            .body(body.to_string().as_bytes().to_vec())
    }

    /// Add a header to the response.
    ///
    /// Header name is lowercased automatically. The value is set as-is.
    ///
    /// # Example
    ///
    /// ```
    /// use rum::http::Response;
    ///
    /// let response = Response::text("don't cache me")
    ///     .header("Cache-Control", "no-cache");
    /// ```
    pub fn header(mut self, name: impl ToString, value: impl ToString) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    /// Send the response to a stream, serialized as bytes.
    pub async fn send(&self, mut stream: impl AsyncWrite + Unpin) -> Result<(), std::io::Error> {
        let mut response = format!("{} {}\r\n", self.version, self.code)
            .as_bytes()
            .to_vec();
        response.extend_from_slice(&self.headers.to_bytes());
        response.extend_from_slice(b"\r\n");
        response.extend_from_slice(&self.body);

        stream.write_all(&response).await
    }

    /// Default not found (404) error.
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

    /// Default method not allowed (405) error.
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

    pub fn not_authorized() -> Self {
        Self::html(
            "
            <h3>
                <center>403 - Not Authorized</center>
            </h3>
            ",
        )
        .code(403)
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

    pub fn unauthorized(auth: &str) -> Self {
        Self::html(
            "
            <h3>
                <center>401 - Unauthorized</center>
            </h3>
            ",
        )
        .code(401)
        .header("www-authenticate", auth)
    }
}
