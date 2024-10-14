use rwf::controller::WebsocketController;
use rwf::http::Server;
use rwf::prelude::*;

mod controllers;
mod models;

/// Index controller serves the index.html template.
#[derive(Default)]
struct IndexController;

#[async_trait]
impl Controller for IndexController {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Response::new().redirect("/signup"))
    }
}

/// TurboStream controller handles WebSocket connections
/// from Turbo's `<turbo-stream-source>`.
#[derive(Default, rwf::macros::WebsocketController)]
struct TurboStreamController;

#[rwf::async_trait]
impl WebsocketController for TurboStreamController {}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Configure logging.
    Logger::init();

    Migrations::migrate().await?;

    Server::new(vec![
        IndexController::default().route("/"),
        TurboStreamController::default().route("/turbo-stream"),
        controllers::signup::SignupController::new().route("/signup"),
        controllers::chat::ChatController::new().route("/chat"),
    ])
    .launch("0.0.0.0:8000")
    .await?;

    Ok(())
}
