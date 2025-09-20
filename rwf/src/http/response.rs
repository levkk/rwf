//! HTTP response.
//!
//! All controllers must return a `Response`.
//!
//! ### Example
//!
//! ```rust
//! use rwf::http::Response;
//!
//! let response = Response::new()
//!     .html("<h1>Hello world!</h1>");
//! ```

use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashMap;
use std::marker::Unpin;
use time::OffsetDateTime;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::{head::Version, Body, Cookie, Cookies, Error, Headers, Request};
use crate::http::encoder::{Encoder, EncodingAlgorithm};
use crate::view::{Template, TurboStream};
use crate::{config::get_config, controller::Session};

static ERROR_TEMPLATE: Lazy<Template> = Lazy::new(|| {
    let template = include_str!("../error.html");
    Template::from_str(template).unwrap()
});

/// Response status, e.g. 404, 200, etc.
#[derive(Debug, Copy, Clone)]
pub enum Status {
    Continue = 100,
    SwitchingProtocols = 101,
    Processing = 102,
    EarlyHints = 103,
    Ok = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultiStatus = 207,
    AlreadyReported = 208,
    IMUsed = 226,
    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    UseProxy = 305,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    PayloadTooLarge = 413,
    URITooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    ImATeapot = 418,
    MisdirectedRequest = 421,
    UnprocessableEntity = 422,
    Locked = 423,
    FailedDependency = 424,
    TooEarly = 425,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HTTPVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NotExtended = 510,
    NetworkAuthenticationRequired = 511,
}

impl Status {
    /// Get HTTP code.
    pub fn code(&self) -> u16 {
        *self as u16
    }

    /// Return true if this is HTTP 200.
    pub fn ok(&self) -> bool {
        self.code() < 300
    }
}
impl From<u16> for Status {
    fn from(code: u16) -> Self {
        match code {
            100 => Status::Continue,
            101 => Status::SwitchingProtocols,
            102 => Status::Processing,
            103 => Status::EarlyHints,

            200 => Status::Ok,
            201 => Status::Created,
            202 => Status::Accepted,
            203 => Status::NonAuthoritativeInformation,
            204 => Status::NoContent,
            205 => Status::ResetContent,
            206 => Status::PartialContent,
            207 => Status::MultiStatus,
            208 => Status::AlreadyReported,
            226 => Status::IMUsed,

            300 => Status::MultipleChoices,
            301 => Status::MovedPermanently,
            302 => Status::Found,
            303 => Status::SeeOther,
            304 => Status::NotModified,
            305 => Status::UseProxy,
            307 => Status::TemporaryRedirect,
            308 => Status::PermanentRedirect,

            400 => Status::BadRequest,
            401 => Status::Unauthorized,
            402 => Status::PaymentRequired,
            403 => Status::Forbidden,
            404 => Status::NotFound,
            405 => Status::MethodNotAllowed,
            406 => Status::NotAcceptable,
            407 => Status::ProxyAuthenticationRequired,
            408 => Status::RequestTimeout,
            409 => Status::Conflict,
            410 => Status::Gone,
            411 => Status::LengthRequired,
            412 => Status::PreconditionFailed,
            413 => Status::PayloadTooLarge,
            414 => Status::URITooLong,
            415 => Status::UnsupportedMediaType,
            416 => Status::RangeNotSatisfiable,
            417 => Status::ExpectationFailed,
            418 => Status::ImATeapot,
            421 => Status::MisdirectedRequest,
            422 => Status::UnprocessableEntity,
            423 => Status::Locked,
            424 => Status::FailedDependency,
            425 => Status::TooEarly,
            426 => Status::UpgradeRequired,
            428 => Status::PreconditionRequired,
            429 => Status::TooManyRequests,
            431 => Status::RequestHeaderFieldsTooLarge,
            451 => Status::UnavailableForLegalReasons,

            500 => Status::InternalServerError,
            501 => Status::NotImplemented,
            502 => Status::BadGateway,
            503 => Status::ServiceUnavailable,
            504 => Status::GatewayTimeout,
            505 => Status::HTTPVersionNotSupported,
            506 => Status::VariantAlsoNegotiates,
            507 => Status::InsufficientStorage,
            508 => Status::LoopDetected,
            510 => Status::NotExtended,
            511 => Status::NetworkAuthenticationRequired,

            // Fallback for unknown status codes
            _ => panic!("Unknown HTTP status code: {}", code),
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
    encoding: EncodingAlgorithm,
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

impl Response {
    /// Create new response with an empty body.
    ///
    /// By default the `Content-Type` header will be set to `text/plain`. This will also
    /// set some other headers to their default values, e.g., `Server` and `Connection`.
    pub fn new() -> Self {
        Self {
            code: 200,
            headers: Headers::from(HashMap::from([
                ("content-type".to_string(), "text/plain".to_string()),
                ("server".to_string(), "rwf".to_string()),
                ("connection".to_string(), "keep-alive".to_string()),
                (
                    "date".to_string(),
                    OffsetDateTime::now_utc()
                        .format(&time::format_description::well_known::Rfc2822)
                        .unwrap(),
                ),
            ])),
            body: Body::bytes(vec![]),
            version: Version::Http1,
            cookies: Cookies::new(),
            session: None,
            encoding: EncodingAlgorithm::Identity,
        }
    }

    /// Create a response from a request. *This is used internally automatically.*
    ///
    /// This makes sure a valid session cookie is set on all responses.
    pub fn from_request(mut self, request: &Request) -> Result<Self, Error> {
        // Session set manually on the request already.
        if let Some(ref session) = self.session {
            self.cookies.add_session(&session)?;
        } else {
            let session = request.session();

            if session.should_renew() || request.renew_session() {
                let session = session
                    .clone()
                    .renew(get_config().general.session_duration());
                self.cookies.add_session(&session)?;

                // Set the session on the response, so it can be
                // passed down in handle_stream.
                self.session = Some(session);
            }
        }

        Ok(self)
    }

    /// Set the request body.
    ///
    /// The body will automatically determine the `Content-Type` and `Content-Length` headers.
    /// If you want to override any of them for some reason, make sure to set them _after_ the body
    /// when building a response.
    pub fn body(mut self, body: impl Into<Body>) -> Self {
        self.body = body.into();
        self.headers
            .insert("content-length".to_string(), self.body.len().to_string());
        self.headers
            .insert("content-type", self.body.mime_type().to_string());
        self
    }

    /// Get response status, e.g. 200 OK.
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
    /// let response = Response::new()
    ///     .text("Created your resource!")
    ///     .code(201);
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
    /// let response = Response::new()
    ///     .json(Body { value: "hello world".to_string() })
    ///    .unwrap();
    /// ```
    pub fn json(self, body: impl Serialize) -> Result<Self, Error> {
        let body = serde_json::to_vec(&body)?;
        Ok(self.body(Body::Json(body, false)))
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
        self.body(Body::Html(body.to_string(), false))
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
        self.body(Body::Text(body.to_string(), false))
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
    /// let response = Response::new()
    ///     .text("don't cache me")
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
        if self.body.is_compressed() {
            match self.encoding {
                EncodingAlgorithm::Gzip => {
                    self.headers.insert("Content-Encoding", "gzip");
                }
                EncodingAlgorithm::Deflate => {
                    self.headers.insert("Content-Encoding", "deflate");
                }
                EncodingAlgorithm::Brotli => {
                    self.headers.insert("Content-Encoding", "brotli");
                }
                EncodingAlgorithm::Identity => {}
            };
            let encoder = Encoder::encoder(self.encoding);
            self.body.send(stream, encoder).await
        } else {
            let encoder = Encoder::encoder(self.encoding);
            self.body.send(stream, encoder).await
        }
    }

    /// Mutable reference to response cookies. Used to set cookies on the response.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::http::{Response, CookieBuilder};
    /// let mut response = Response::new();
    /// response
    ///     .cookies()
    ///     .add(
    ///         CookieBuilder::new()
    ///             .name("rwf_aid")
    ///             .value("1234")
    ///             .build()
    ///     );
    /// ```
    pub fn cookies(&mut self) -> &mut Cookies {
        &mut self.cookies
    }

    /// Set a private (encrypted) cookie on the response.
    ///
    /// Works exactly the same way as [`Response::cookie`].
    pub fn private_cookie(mut self, cookie: Cookie) -> Result<Self, Error> {
        self.cookies.add_private(cookie)?;
        Ok(self)
    }

    /// Set a cookie on the response.
    ///
    /// This is more ergonomic that using [`Request::cookies`].
    ///
    /// ```
    /// # use rwf::http::{Response, CookieBuilder};
    /// let response = Response::new()
    ///     .cookie(
    ///         CookieBuilder::new()
    ///             .name("my_cookie")
    ///             .value("rwf")
    ///             .build()
    ///     );
    /// ```
    pub fn cookie(mut self, cookie: Cookie) -> Self {
        self.cookies.add(cookie);
        self
    }

    /// Set the session on the response.
    ///
    /// The session is renewed automatically if it has expired.
    pub fn set_session(mut self, session: Session) -> Self {
        self.session = Some(session);
        self
    }

    /// Get the response session, if any is set.
    ///
    /// All requests should have a session unless the browser
    /// doesn't respect cookies.
    pub fn session(&self) -> &Option<Session> {
        &self.session
    }

    /// Response is an agreement to upgrade the connection to use the WebSocket protocol.
    pub fn websocket_upgrade(&self) -> bool {
        self.code == 101 && self.headers.get("upgrade").map(|s| s == "websocket") == Some(true)
    }

    /// Create a response containing turbo streams. This sets the correct
    /// `Content-Type` headers to be parsed by Turbo.
    pub fn turbo_stream(self, body: &[TurboStream]) -> Self {
        let body = body
            .iter()
            .map(|b| b.clone().render())
            .collect::<Vec<_>>()
            .join("\n");
        self.html(body)
            .header("content-type", "text/vnd.turbo-stream.html")
    }

    /// Create a `404 - Not Found` response.
    pub fn not_found() -> Self {
        Self::error_pretty("404 - Not Found", "").code(404)
    }

    /// Create a `405 - Method Not Allowed` response.
    pub fn method_not_allowed() -> Self {
        Self::error_pretty("405 - Method Not Allowed", "").code(405)
    }

    /// Create a `400 - Bad Request` response.
    pub fn bad_request() -> Self {
        Self::error_pretty("400 - Bad Request", "").code(400)
    }

    /// Create CSRF token validation error. Returns `400 - Bad Request` response.
    pub fn csrf_error() -> Self {
        Self::error_pretty(
            "400 - CSRF Token Validation Failed",
            "The supplied CSRF token is not valid. Reload the page to get a new one.",
        )
        .code(400)
    }

    /// Create `501 - Not Implemented` response.
    pub fn not_implemented() -> Self {
        Self::error_pretty("501 - Not Implemented", "").code(501)
    }

    /// Create `403 - Forbidden` response.
    pub fn forbidden() -> Self {
        Self::error_pretty("403 - Forbidden", "").code(403)
    }

    /// Create `413 - Content Too Large` response.
    pub fn content_too_large() -> Self {
        Self::error_pretty("413 - Content Too Large", "").code(413)
    }

    /// Create `500 - Internal Server Error` response.
    ///
    /// Requires the error that was returned for debugging purposes.
    /// The error is shown in development (debug) and hidden in production (release).
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

    /// Use the internal template to render a better looking error page.
    /// Returns `500 - Internal Server Error` response.
    pub fn error_pretty(title: &str, message: &str) -> Self {
        let body = ERROR_TEMPLATE
            .render([("title", title), ("message", message)])
            .unwrap();

        Self::new().html(body).code(500)
    }

    /// Create `401 - Unauthorized` response.
    pub fn unauthorized(auth: Option<&str>) -> Self {
        let response = Self::error_pretty("401 - Unauthorized", "").code(401);
        match auth {
            Some(auth) => response.header("www-authenticate", auth),
            None => response,
        }
    }

    /// Create `429 - Too Many` response.
    pub fn too_many() -> Self {
        Self::error_pretty("429 - Too Many", "").code(429)
    }

    /// Create `302 - Found` response, also known as a redirect.
    pub fn redirect(self, to: impl ToString) -> Self {
        self.html("")
            .header("location", to)
            .code(302)
            .header("content-length", 0)
            .header("cache-control", "no-cache")
    }

    /// Create `101 - Switching Protocols`. Can be used for upgrading the connection
    /// to HTTP/2 or WebSocket. The protocol argument isn't checked, so ideally this is used
    /// internally only.
    pub fn switching_protocols(protocol: &str) -> Self {
        let mut response = Self::default();
        response.headers.clear();
        response
            .header("connection", "upgrade")
            .header("upgrade", protocol)
            .code(101)
    }

    /// Response headers.
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Mutable response headers.
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Compresses the http response specified by the [EncodingAlgorithm].
    pub fn compress(mut self, algorithm: EncodingAlgorithm) -> Result<Self, Error> {
        self.encoding = algorithm;
        self.body.enable_compression();
        Ok(self)
    }
}

impl From<serde_json::Value> for Response {
    fn from(value: serde_json::Value) -> Response {
        Response::new().json(value).unwrap()
    }
}

impl From<String> for Response {
    fn from(value: String) -> Response {
        Response::new().html(value)
    }
}

impl From<&[TurboStream]> for Response {
    fn from(value: &[TurboStream]) -> Response {
        Response::new().turbo_stream(value)
    }
}

impl From<Vec<TurboStream>> for Response {
    fn from(value: Vec<TurboStream>) -> Response {
        Response::new().turbo_stream(&value)
    }
}
