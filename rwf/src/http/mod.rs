//! HTTP protocol.
//!
//! Rwf comes with its own implementation, including routing of requests,
//! and a tokio-based async server that can handle millions of
//! concurrent connections.
//!
//! ##### Support for HTTP/2
//! Currently, only HTTP/1.1 is supported. Support for HTTP/2 is a work in progress.
//! You can put the Rwf application behind a load balancer (like nginx) that supports
//! HTTP/2 to take advantage of its performance enhancements.
#![allow(dead_code)]
pub mod authorization;
pub mod body;
pub mod cookies;
pub mod error;
pub mod form;
pub mod form_data;
pub mod handler;
pub mod head;
pub mod headers;
pub mod path;
pub mod request;
pub mod response;
pub mod router;
pub mod server;
pub mod url;
pub mod websocket;

#[cfg(feature = "wsgi")]
pub mod wsgi;

pub use authorization::Authorization;
pub use body::Body;
pub use cookies::{Cookie, CookieBuilder, Cookies};
pub use error::Error;
pub use form::{Form, FromFormData};
pub use form_data::FormData;
pub use handler::Handler;
pub use head::{Head, Method};
pub use headers::Headers;
pub use path::{Params, Path, Query, ToParameter};
pub use request::Request;
pub use response::Response;
pub use router::Router;
pub use server::{Server, Stream};
pub use url::{urldecode, urlencode};
pub use websocket::{Message, ToMessage};

/// HTTP protocol kind.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Protocol {
    /// HTTP/1.1
    Http1,
    /// HTTP/2
    Http2,
    /// WebSocket
    Websocket,
}
