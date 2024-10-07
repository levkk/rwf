use rum::controller::WebsocketController;
use rum::http::Server;
use rum::job::{Error as JobError, Worker};
use rum::prelude::*;

use rand::Rng;
use serde::{Deserialize, Serialize};
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

#[derive(Default)]
struct CanvasController;

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
        let args = ExpensiveJob {
            session_id: request.session_id(),
        };

        ExpensiveJob::default()
            .execute_async(serde_json::to_value(args)?)
            .await?;

        let message = format!(
            "This was updated by Turbo, random number of the day is: {}",
            rand::thread_rng().gen_range(0..25)
        );

        let turbo_stream = CanvasController::canvas(message).await?;

        Ok(Response::new().turbo_stream(turbo_stream))
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct ExpensiveJob {
    session_id: Option<SessionId>,
}

#[rum::async_trait]
impl Job for ExpensiveJob {
    async fn execute(&self, args: serde_json::Value) -> Result<(), JobError> {
        let args: Self = serde_json::from_value(args)?;

        if let Some(ref session_id) = args.session_id {
            let message = "I just did an expensive job.";

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
