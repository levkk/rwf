use rum::controller::WebsocketController;
use rum::http::Server;
use rum::prelude::*;

use rand::Rng;
use tracing::info;

#[derive(Default)]
struct IndexController;

#[async_trait]
impl Controller for IndexController {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Response::new().html(
            Template::cached("templates/index.html")
                .await?
                .render_default()?,
        ))
    }
}

#[derive(Default)]
struct TurboStreamController;

#[rum::async_trait]
impl WebsocketController for TurboStreamController {
    async fn client_message(&self, _client_id: &SessionId, message: Message) -> Result<(), Error> {
        info!(
            "ignoring {:?} from client, turbo doesn't use websockets to send messages to server",
            message
        );

        Ok(())
    }
}

#[rum::async_trait]
impl Controller for TurboStreamController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        WebsocketController::handle(self, request).await
    }
}

#[derive(rum::macros::Context)]
struct Canvas {
    body: String,
}

#[derive(Default)]
struct CanvasController;

#[rum::async_trait]
impl Controller for CanvasController {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        let canvas = Template::cached("templates/canvas.html").await?;
        let message = format!(
            "This was updated by Turbo, random number of the day is: {}",
            rand::thread_rng().gen_range(0..25)
        );

        let body = canvas.render(&Canvas { body: message }.try_into()?)?;

        Ok(Response::new().turbo_stream(TurboStream::new(body).action("replace").target("canvas")))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    Logger::init();

    Server::new(vec![
        IndexController::default().route("/"),
        TurboStreamController::default().route("/turbo-stream"),
        CanvasController::default().route("/update-canvas"),
    ])
    .launch("0.0.0.0:8000")
    .await?;

    Ok(())
}
