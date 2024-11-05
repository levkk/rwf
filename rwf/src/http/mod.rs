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
#[cfg(not(feature = "cloudflare"))]
pub mod server;
pub mod url;
pub mod websocket;

#[cfg(feature = "wsgi")]
pub mod wsgi;

#[cfg(feature = "cloudflare")]
pub mod cloudflare;

use std::marker::PhantomData;

pub use authorization::Authorization;
pub use body::Body;
pub use cookies::{Cookie, CookieBuilder, Cookies};
pub use error::Error;
pub use form::{Form, FromFormData};
pub use form_data::FormData;
pub use handler::Handler;
pub use head::{Head, Method, Version};
pub use headers::Headers;
pub use path::{Params, Path, Query, ToParameter};
pub use request::Request;
pub use response::Response;
pub use router::Router;
#[cfg(not(feature = "cloudflare"))]
pub use server::{Server, Stream};

#[cfg(feature = "cloudflare")]
pub enum Stream<'a> {
    Cloudflare { _unused: &'a str },
}

pub use url::{urldecode, urlencode};
pub use websocket::Message;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Protocol {
    Http1,
    Http2,
    Websocket,
}
