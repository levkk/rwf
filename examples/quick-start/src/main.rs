use rum::http::Server;
use rum::logging::setup_logging;
use rum::prelude::*;

#[derive(Default)]
struct IndexController;

#[async_trait]
impl Controller for IndexController {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Response::new().html("<h1>Hey Rum!</h1>"))
    }
}

#[tokio::main]
async fn main() {
    setup_logging();

    Server::new(vec![IndexController::default().route("/")])
        .launch("0.0.0.0:8000")
        .await
        .expect("error shutting down server");
}
