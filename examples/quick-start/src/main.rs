use rwf::http::{self, Server};
use rwf::prelude::*;

#[derive(Default)]
struct Index;

#[async_trait]
impl Controller for Index {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::new().html("<h1>My first Rwf app!</h1>"))
    }
}

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    Logger::init();

    Server::new(vec![route!("/" => Index)])
        .launch("0.0.0.0:8000")
        .await
}
