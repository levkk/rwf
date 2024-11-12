mod controllers;
mod models;

use rwf::http::{self, Server};
use rwf::prelude::*;

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    Logger::init();

    Server::new(vec![route!("/" => controllers::Upload)])
        .launch("0.0.0.0:8000")
        .await
}
