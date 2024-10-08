use rum::controller::WebsocketController;
use rum::http::Server;
use rum::job::{Error as JobError, Worker};
use rum::prelude::*;

use serde::{Deserialize, Serialize};
use time::Duration;

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

/// Index controller serves the index.html template.
#[derive(Default)]
struct IndexController;

#[async_trait]
impl Controller for IndexController {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Template::cached_static("templates/index.html").await?)
    }
}

/// TurboStream controller handles WebSocket connections
/// from Turbo's `<turbo-stream-source>`.
#[derive(Default, rum::macros::WebsocketController)]
struct TurboStreamController;

#[rum::async_trait]
impl WebsocketController for TurboStreamController {
    /// Update the page when the WebSocket connection is established.
    /// We don't need to do this, but it's fun to show that WebSockets are working
    /// with Turbo.
    async fn client_connected(&self, session_id: &SessionId) -> Result<(), Error> {
        let message = "Turbo Stream connected via WebSocket";

        if let Ok(page) = PageController::page(message).await {
            Comms::websocket(session_id).send(Message::turbo_stream(page))?;
        }

        Ok(())
    }
}

/// Draw on the page using only Turbo Streams.
#[derive(rum::macros::Context)]
struct Page {
    body: String,
}

struct PageController {
    // This field is updated for all HTTP requests from all clients.
    // Try clicking the button from different browser windows
    // (+ incognito mode to have a different session).
    counter: Arc<AtomicUsize>,
}

impl Default for PageController {
    fn default() -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(1)),
        }
    }
}

impl PageController {
    /// Generate an HTML template and send it as a TurboStream `<turbo-stream>`
    /// HTML element.
    async fn page(message: impl ToString) -> Result<TurboStream, Error> {
        let page = Template::cached("templates/page.html").await?;
        let body = page.render(
            &Page {
                body: message.to_string(),
            }
            .try_into()?,
        )?;
        Ok(TurboStream::new(body).action("replace").target("page"))
    }
}

#[rum::async_trait]
impl Controller for PageController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        // Count clicks app-wide.
        let click = self.counter.fetch_add(1, Ordering::Relaxed);

        // Run a job in the background.
        let args = ExpensiveJob {
            session_id: request.session_id(),
            click,
        };

        // This runs on the background worker and doesn't block the HTTP request.
        ExpensiveJob::default()
            .execute_delay(serde_json::to_value(args)?, Duration::seconds(2))
            .await?;

        let message = format!("This button was clicked {} times", click);

        let turbo_stream = PageController::page(message).await?;

        // Update page via Turbo Stream response.
        Ok(Response::new().turbo_stream(turbo_stream))
    }
}

// Just a background job. It can send an email or
// calculate nth digit of Pi, but whatever it is, it will
// happen in the background and the result will be sent asynchronously
// via WebSocket to the client.
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

            if let Ok(page) = PageController::page(message).await {
                Comms::websocket(session_id).send(Message::turbo_stream(page))?;
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load Rum config file.
    Config::load().await?;

    // Configure logging.
    Logger::init();

    Migrations::migrate().await?;

    // Start a background worker.
    Worker::new(vec![ExpensiveJob::default().job()])
        .start()
        .await?;

    Server::new(vec![
        IndexController::default().route("/"),
        TurboStreamController::default().route("/turbo-stream"),
        PageController::default().route("/update-page"),
    ])
    .launch("0.0.0.0:8000")
    .await?;

    Ok(())
}
