use rwf::http::{self, Server};
use rwf::prelude::*;

#[derive(Default)]
struct Index;

#[async_trait]
impl Controller for Index {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::new().html("<h1>My first Rwf app!</h1>"))
    }
}

#[derive(Default)]
struct CurrentTime;

#[async_trait]
impl Controller for CurrentTime {
    /// This function responds to all incoming HTTP requests.
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let time = OffsetDateTime::now_utc();
        println!("{:?}", request.headers().get("accept"));

        // This creates an HTTP "200 OK" response,
        // with "Content-Type: text/plain" header.
        let response = Response::new().text(format!("The current time is: {:?}", time));

        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    Logger::init();

    Server::new(vec![route!("/" => Index), route!("/time" => CurrentTime)])
        .launch("0.0.0.0:8001")
        .await
}
