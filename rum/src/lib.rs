pub mod controller;
pub mod http;
pub mod job;
pub mod logging;
pub mod model;
pub mod server;
pub mod view;

pub use async_trait::async_trait;
pub use rum_macros as macros;
pub use tokio_postgres;

pub use http::Server;

#[allow(dead_code)]
fn snake_case(string: &str) -> String {
    let mut result = "".to_string();

    for (i, c) in string.chars().enumerate() {
        if c.is_ascii_uppercase() && i != 0 {
            result.push('_');
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }

    result
}
