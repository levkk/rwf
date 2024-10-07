use rum::controller::WebsocketController;
use rum::http::Server;
use rum::job::{Error as JobError, Worker};
use rum::prelude::*;

use serde::{Deserialize, Serialize};
use time::Duration;
use tracing::info;

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Default)]
struct IndexController;

#[async_trait]
impl Controller for IndexController {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Template::cached_static("templates/index.html").await?)
    }
}

#[derive(Default, rum::macros::WebsocketController)]
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

#[derive(rum::macros::Context)]
struct Canvas {
    body: String,
}

struct CanvasController {
    counter: Arc<AtomicUsize>,
}

impl Default for CanvasController {
    fn default() -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(1)),
        }
    }
}

impl CanvasController {
    async fn canvas(message: impl ToString) -> Result<TurboStream, Error> {
        let canvas = Template::cached("templates/canvas.html").await?;
        let body = canvas.render(
            &Canvas {
                body: message.to_string(),
            }
            .try_into()?,
        )?;
        Ok(TurboStream::new(body).action("replace").target("canvas"))
    }
}

#[rum::async_trait]
impl Controller for CanvasController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let click = self.counter.fetch_add(1, Ordering::Relaxed);

        let args = ExpensiveJob {
            session_id: request.session_id(),
            click,
        };

        ExpensiveJob::default()
            .execute_delay(serde_json::to_value(args)?, Duration::seconds(2))
            .await?;

        let message = format!("This button was clicked {} times", click,);

        let turbo_stream = CanvasController::canvas(message).await?;

        Ok(Response::new().turbo_stream(turbo_stream))
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct ExpensiveJob {
    session_id: Option<SessionId>,
    click: usize,
}

#[rum::async_trait]
impl Job for ExpensiveJob {
    async fn execute(&self, args: serde_json::Value) -> Result<(), JobError> {
        let args: Self = serde_json::from_value(args)?;

        if let Some(ref session_id) = args.session_id {
            let message = format!(
                "Button clicked {} times, delivered via WebSocket from a background job.",
                args.click
            );

            if let Ok(canvas) = CanvasController::canvas(message).await {
                Comms::websocket(session_id).send(Message::turbo_stream(canvas))?;
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    Logger::init();
    Config::load().await?;

    Worker::new(vec![ExpensiveJob::default().job()])
        .start()
        .await?;

    Server::new(vec![
        IndexController::default().route("/"),
        TurboStreamController::default().route("/turbo-stream"),
        CanvasController::default().route("/update-canvas"),
    ])
    .launch("0.0.0.0:8000")
    .await?;

    Ok(())
}
