use rwf::prelude::*;

#[controller]
async fn index() -> Response {
    Response::new().html("<h1>My first Rwf app!</h1>")
}

use rwf::http::{self, Server};

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    // Configure then logger (stderr with colors by default)
    Logger::init();

    Server::new(vec![route!("/" => index)])
        .launch("0.0.0.0:8000")
        .await
}
