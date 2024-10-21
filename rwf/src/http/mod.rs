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
pub use url::urldecode;
pub use websocket::Message;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Protocol {
    Http1,
    Http2,
    Websocket,
}
