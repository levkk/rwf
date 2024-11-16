//! Rwf is a comprehensive framework for building web applications in Rust. Written using the classic MVC pattern
//! (model-view-controller), Rwf comes standard with everything you need to easily build fast and secure web apps.
//!
//! This documentation serves primarily as a reference for methods and types provided by this
//! and [`rwf_macros`] crates. For user guides, refer to the [documentation here](https://levkk.github.io/rwf/).
//!
//! # Getting started
//!
//! Rwf is a Rust library built on top of Tokio, and can be added to any binary or library Rust project:
//!
//! ```bash
//! cargo add rwf
//! cargo add tokio@1 --features full
//! ```
//!
//! Rwf has many types and traits that make it ergonomic. You can include them all with just one import:
//!
//! ```
//! use rwf::prelude::*;
//! ```
//!
//! While not required, this makes things simpler.
//!
//! ### Controllers
//!
//! Rwf is an MVC framework, so **C**ontrollers are fundamental to serving HTTP requests. Defining controllers requires
//! imlementing the [`controller::Controller`] trait for a struct:
//!
//! ```rust
//! use rwf::prelude::*;
//!
//! #[derive(Default)]
//! struct Index;
//!
//! #[rwf::async_trait]
//! impl Controller for Index {
//!     async fn handle(&self, request: &Request) -> Result<Response, Error> {
//!         Ok(Response::new().html("<h1>Hello from Rwf!</h1>"))
//!     }
//! }
//! ```
//!
//! Most Rwf traits are asynchronous and use the `async_trait` crate to make it user-friendly.
//!
//! ### HTTP server
//!
//! Launching the Rwf HTTP server requires mapping routes to controllers, and can be done at application startup:
//!
//! ```rust
//! use rwf::http::Server;
//!
//! # use rwf::prelude::*;
//! # #[derive(Default)]
//! # struct Index;
//! #
//! # #[rwf::async_trait]
//! # impl Controller for Index {
//! #    async fn handle(&self, request: &Request) -> Result<Response, Error> {
//! #        Ok(Response::new().html("<h1>Hello from Rwf!</h1>"))
//! #    }
//! # }
//! let server = Server::new(vec![
//!     route!("/" => Index),
//! ]);
//! ```
//!
//! With all the routes mapped to controllers, you can launch the server from anywhere in your app. Typically though,
//! this is done from the main function:
//!
//! ```rust,ignore
//! use rwf::http::{Server, self};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), http::Error> {
//!     Server::new(vec![
//!         route!("/" => Index),
//!     ])
//!     .launch("0.0.0.0:8000")
//!     .await
//! }
//! ```
//!
pub mod analytics;
pub mod colors;
pub mod comms;
pub mod config;
pub mod controller;
pub mod crypto;
pub mod error;
pub mod hmr;
pub mod http;
pub mod job;
pub mod lock;
pub mod logging;
pub mod model;
pub mod prelude;
pub mod view;

/// Wrapper around async traits to make them easy to use.
pub use async_trait::async_trait;
/// Rwf macros that help reduce boilerplate code.
pub use rwf_macros as macros;
/// Serde is used for (de)serialization.
pub use serde;
/// Tokio is an asynchronous runtime for Rust.
pub use tokio;
/// Asynchronous PostgreSQL driver.
pub use tokio_postgres;

use std::net::SocketAddr;

/// Convert text to snake_case.
pub fn snake_case(string: &str) -> String {
    let mut result = "".to_string();

    for (i, c) in string.chars().enumerate() {
        if c.is_ascii_uppercase() && i != 0 {
            result.push('_');
            result.push(c.to_ascii_lowercase());
        } else if c == '-' {
            result.push('_');
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }

    result
}

/// Convert the first letter of the stirng to uppercase lettering.
pub fn capitalize(string: &str) -> String {
    let mut iter = string.chars();
    let uppercase = match iter.next() {
        None => String::new(),
        Some(letter) => letter.to_uppercase().chain(iter).collect(),
    };

    uppercase
}

/// Convert string to PascalCase (often confused with camelCase).
pub fn pascal_case(string: &str) -> String {
    string
        .split("_")
        .map(|s| capitalize(s))
        .collect::<Vec<_>>()
        .join("")
}

/// Remove unsafe characters from a string printed
/// inside an HTML template.
pub fn safe_html(string: &str) -> String {
    string.replace("<", "&lt;").replace(">", "&gt;")
}

/// Extract the first socket address from a string.
pub fn peer_addr(addr: &str) -> Option<SocketAddr> {
    use std::net::ToSocketAddrs;

    if let Ok(mut iter) = addr.to_socket_addrs() {
        if let Some(addr) = iter.next() {
            return Some(addr.clone());
        }
    }

    None
}
