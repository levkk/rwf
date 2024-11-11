//! HTTP response.

use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashMap;
use std::marker::Unpin;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::{head::Version, Body, Cookie, Cookies, Error, Headers, Request};
use crate::view::{Template, TurboStream};
use crate::{config::get_config, controller::Session};

static ERROR_TEMPLATE: Lazy<Template> = Lazy::new(|| {
    let template = include_str!("error.html");
    Template::from_str(template).unwrap()
});

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

    pub fn ok(&self) -> bool {
        self.code() < 300
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
#[derive(Debug)]
pub struct Response {
    code: u16,
    headers: Headers,
    version: Version,
    body: Body,
    cookies: Cookies,
    session: Option<Session>,
}

impl Default for Response {
    fn default() -> Self {
        Self {
            code: 200,
            headers: Headers::from(HashMap::from([
                ("content-type".to_string(), "text/plain".to_string()),
                ("server".to_string(), "rwf".to_string()),
                ("connection".to_string(), "keep-alive".to_string()),
            ])),
            body: Body::bytes(vec![]),
            version: Version::Http1,
            cookies: Cookies::new(),
            session: None,
        }
    }
}

impl Response {
    /// Create empty response.
    pub fn new() -> Self {
        Self {
            code: 200,
            headers: Headers::from(HashMap::from([
                ("content-type".to_string(), "text/plain".to_string()),
                ("server".to_string(), "rwf".to_string()),
                ("connection".to_string(), "keep-alive".to_string()),
            ])),
            body: Body::bytes(vec![]),
            version: Version::Http1,
            cookies: Cookies::new(),
            session: None,
        }
    }

    /// Create a response from a request.
    pub fn from_request(mut self, request: &Request) -> Result<Self, Error> {
        // Set an anonymous session if none is set on the request.
        if self.session.is_none() && request.session().is_none() {
            self.session = Some(Session::anonymous());
        }

        // Session set manually on the request already.
        if let Some(ref session) = self.session {
            self.cookies.add_session(&session)?;
        } else {
            let session = request.session();

            if let Some(session) = session {
                if !session.expired() {
                    let session = session
                        .clone()
                        .renew(get_config().general.session_duration());
                    self.cookies.add_session(&session)?;

                    // Set the session on the response, so it can be
                    // passed down in handle_stream.
                    self.session = Some(session);
                }
            }
        }

        Ok(self)
    }

    pub fn body(mut self, body: impl Into<Body>) -> Self {
        self.body = body.into();
        self.headers
            .insert("content-length".to_string(), self.body.len().to_string());
        self.headers
            .insert("content-type", self.body.mime_type().to_string());
        self
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
    /// use rwf::http::Response;
    ///
    /// let response = Response::new().text("OK").code(200);
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
    /// use rwf::http::Response;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Body {
    ///     value: String,
    /// }
    ///
    /// let response = Response::new().json(Body { value: "hello world".to_string() })
    ///    .unwrap()
    ///    .code(200);
    /// ```
    pub fn json(self, body: impl Serialize) -> Result<Self, Error> {
        let body = serde_json::to_vec(&body)?;
        Ok(self.body(Body::Json(body)))
    }

    /// Create a response with an HTML body.
    ///
    /// # Example
    ///
    /// ```
    /// use rwf::http::Response;
    ///
    /// let response = Response::new().html("<h1>Hello world</h1>");
    /// ```
    pub fn html(self, body: impl ToString) -> Self {
        self.body(Body::Html(body.to_string()))
    }

    /// Create a response with a plain text body.
    ///
    /// # Example
    ///
    /// ```
    /// use rwf::http::Response;
    ///
    /// let response = Response::new().text("Hello world");
    /// ```
    pub fn text(self, body: impl ToString) -> Self {
        self.body(Body::Text(body.to_string()))
    }

    /// Add a header to the response.
    ///
    /// Header name is lowercased automatically. The value is set as-is.
    ///
    /// # Example
    ///
    /// ```
    /// use rwf::http::Response;
    ///
    /// let response = Response::default().text("don't cache me")
    ///     .header("Cache-Control", "no-cache");
    /// ```
    pub fn header(mut self, name: impl ToString, value: impl ToString) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    /// Send the response to a stream, serialized as bytes.
    pub async fn send(mut self, mut stream: impl AsyncWrite + Unpin) -> Result<(), std::io::Error> {
        let mut response = format!("{} {}\r\n", self.version, self.code)
            .as_bytes()
            .to_vec();

        response.extend_from_slice(&self.headers.to_bytes());
        response.extend_from_slice(&self.cookies.to_headers());
        response.extend_from_slice(b"\r\n");

        stream.write_all(&response).await?;
        self.body.send(stream).await
    }

    /// Mutable reference to response cookies.
    pub fn cookies(&mut self) -> &mut Cookies {
        &mut self.cookies
    }

    /// Set a private (encrypted) cookie on the response.
    pub fn private_cookie(mut self, cookie: Cookie) -> Result<Self, Error> {
        self.cookies.add_private(cookie)?;
        Ok(self)
    }

    /// Set a cookie on the response.
    pub fn cookie(mut self, cookie: Cookie) -> Self {
        self.cookies.add(cookie);
        self
    }

    pub fn set_session(mut self, session: Session) -> Self {
        self.session = Some(session);
        self
    }

    pub fn session(&self) -> &Option<Session> {
        &self.session
    }

    pub fn websocket_upgrade(&self) -> bool {
        self.code == 101 && self.headers.get("upgrade").map(|s| s == "websocket") == Some(true)
    }

    pub fn turbo_stream(self, body: &[TurboStream]) -> Self {
        let body = body
            .iter()
            .map(|b| b.clone().render())
            .collect::<Vec<_>>()
            .join("\n");
        self.html(body)
            .header("content-type", "text/vnd.turbo-stream.html")
    }

    /// Default not found (404) error.
    pub fn not_found() -> Self {
        Self::error_pretty("404 - Not Found", "").code(404)
    }

    /// Default method not allowed (405) error.
    pub fn method_not_allowed() -> Self {
        Self::error_pretty("405 - Method Not Allowed", "").code(405)
    }

    pub fn bad_request() -> Self {
        Self::error_pretty("400 - Bad Request", "").code(400)
    }

    pub fn csrf_error() -> Self {
        Self::error_pretty(
            "400 - CSRF Token Validation Failed",
            "The supplied CSRF token is not valid. Reload the page to get a new one.",
        )
        .code(400)
    }

    pub fn not_implemented() -> Self {
        Self::error_pretty("501 - Not Implemented", "").code(501)
    }

    pub fn forbidden() -> Self {
        Self::error_pretty("403 - Forbidden", "").code(403)
    }

    pub fn content_too_large() -> Self {
        Self::error_pretty("413 - Content Too Large", "").code(413)
    }

    pub fn internal_error(err: impl std::error::Error) -> Self {
        // TODO:
        #[cfg(debug_assertions)]
        let err = format!("{}", err);

        #[cfg(not(debug_assertions))]
        let err = {
            let _ = err;
            ""
        };

        Self::error_pretty("500 - Internal Server Error", &err)
    }

    pub fn error_pretty(title: &str, message: &str) -> Self {
        let body = ERROR_TEMPLATE
            .render([("title", title), ("message", message)])
            .unwrap();

        Self::new().html(body).code(500)
    }

    pub fn unauthorized(auth: &str) -> Self {
        Self::error_pretty("401 - Unauthorized", "")
            .code(401)
            .header("www-authenticate", auth)
    }

    pub fn too_many() -> Self {
        Self::error_pretty("429 - Too Many", "").code(429)
    }

    pub fn redirect(self, to: impl ToString) -> Self {
        self.html("")
            .header("location", to)
            .code(302)
            .header("content-length", 0)
            .header("cache-control", "no-cache")
    }

    pub fn switching_protocols(protocol: &str) -> Self {
        let mut response = Self::default();
        response.headers.clear();
        response
            .header("connection", "upgrade")
            .header("upgrade", protocol)
            .code(101)
    }
}
