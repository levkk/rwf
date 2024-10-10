use rwf::http::Server;
use rwf::prelude::*;

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
    Logger::init();

    Server::new(vec![IndexController::default().route("/")])
        .launch("0.0.0.0:8000")
        .await
        .expect("error shutting down server");
}
