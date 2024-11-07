use rwf::controller::rack::RackController;
use rwf::http::{self, Server};
use rwf::prelude::*;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), http::Error> {
    Logger::init();

    let controller = RackController::new("todo");

    Server::new(vec![controller.wildcard("/")])
        .launch("0.0.0.0:8000")
        .await
}
