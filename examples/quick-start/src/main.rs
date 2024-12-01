use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use rwf::http::{self, Server};
use rwf::prelude::*;

/// Simple controller.
#[controller]
async fn index() -> Response {
    Response::new().html("<h1>My first Rwf app!</h1>")
}

/// Controller handler processing data from the request.
#[controller]
async fn with_request(request: &Request) -> Response {
    let name = request
        .query()
        .get::<String>("name")
        .unwrap_or(String::from(""));

    let body = format!("<h2>Name: {name}</h2>");

    Response::new().html(body)
}

/// Controller handler that could return an error.
#[controller]
async fn with_error(request: &Request) -> Result<Response, Error> {
    let name = request.query().get_required::<String>("name")?;
    let body = format!("<h2>Name: {name}</h2>");

    Ok(Response::new().html(body))
}

/// More complex controller with internal state.
struct RequestCount {
    requests: Arc<AtomicUsize>,
}

// `route!` macro uses `Self::default()` to instantiate your controllers.
impl Default for RequestCount {
    fn default() -> Self {
        Self {
            requests: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[async_trait]
impl Controller for RequestCount {
    /// This function responds to incoming HTTP requests to this controller.
    async fn handle(&self, _req: &Request) -> Result<Response, Error> {
        let time = OffsetDateTime::now_utc();
        let requests = self.requests.fetch_add(1, Ordering::Relaxed) + 1;

        // This creates an HTTP "200 OK" response,
        // with "Content-Type: text/plain" header.
        let response = Response::new().html(format!(
            "The current time is: {time:?} <br> Requests received: {requests}"
        ));

        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    Logger::init();

    Server::new(vec![
        route!("/" => index),
        route!("/time" => RequestCount),
        route!("/with-request" => with_request),
        route!("/with-error" => with_error),
    ])
    .launch()
    .await
}
