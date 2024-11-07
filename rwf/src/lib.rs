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

pub use async_trait::async_trait;
pub use rwf_macros as macros;
pub use serde;
pub use tokio;
pub use tokio_postgres;

pub use controller::{Controller, Error, ModelController, RestController};
pub use http::Server;
pub use logging::Logger;

use std::net::SocketAddr;

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

pub fn capitalize(string: &str) -> String {
    let mut iter = string.chars();
    let uppercase = match iter.next() {
        None => String::new(),
        Some(letter) => letter.to_uppercase().chain(iter).collect(),
    };

    uppercase
}

pub fn pascal_case(string: &str) -> String {
    string
        .split("_")
        .map(|s| capitalize(s))
        .collect::<Vec<_>>()
        .join("")
}

pub fn peer_addr(addr: &str) -> Option<SocketAddr> {
    use std::net::ToSocketAddrs;

    if let Ok(mut iter) = addr.to_socket_addrs() {
        if let Some(addr) = iter.next() {
            return Some(addr.clone());
        }
    }

    None
}
