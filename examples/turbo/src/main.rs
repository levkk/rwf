use rwf::controller::{StaticFiles, WebsocketController};
use rwf::http::Server;
use rwf::prelude::*;

mod controllers;
mod models;

use controllers::*;

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
    Template::defaults(context!(
        "rwf_stimulus_src" => "/static/js/stimulus.js",
        "rwf_turbo_src" => "/static/js/turbo.js",
    ));

    // Configure logging.
    Logger::init();

    Migrations::migrate().await?;

    Server::new(vec![
        route!("/" => IndexController),
        route!("/turbo-stream" => TurboStreamController),
        route!("/signup" => SignupController),
        route!("/logout" => LogoutController),
        route!("/chat" => ChatController),
        route!("/chat/typing" => TypingController),
        StaticFiles::serve("static")?,
    ])
    .launch("0.0.0.0:8000")
    .await?;

    Ok(())
}
