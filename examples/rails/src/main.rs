use rwf::controller::RackController;
use rwf::http::{self, Server};
use rwf::prelude::*;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), http::Error> {
    Logger::init();

    let controller = RackController::new("todo");

    Server::new(vec![route!("/rust" => Index), controller.wildcard("/")])
        .launch()
        .await
}

#[derive(Default)]
struct Index;

#[async_trait]
impl Controller for Index {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Response::new().html("<h1>This is served by Rust!</h1>"))
    }
}
