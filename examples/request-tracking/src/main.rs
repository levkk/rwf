mod controllers;
mod models;

use rand::Rng;
use rwf::{
    http::{self, Server},
    prelude::*,
};

#[derive(Default)]
struct Index;

#[async_trait]
impl Controller for Index {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        let ok = rand::thread_rng().gen::<bool>();

        if ok {
            // This is tracked.
            Ok(Response::new().html("
                <h2>All requests are tracked</h2>
                <p>To view requests, connect to the <code>rwf_request_tracking</code> database and run:</p>
                <code>SELECT * FROM rwf_requests ORDER BY id</code>
            "))
        } else {
            // This is tracked also.
            Err(Error::HttpError(Box::new(http::Error::MissingParameter)))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    Logger::init();
    Migrations::migrate().await?;

    Server::new(vec![route!("/" => Index)]).launch().await
}
