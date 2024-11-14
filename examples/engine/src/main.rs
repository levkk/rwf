use rwf::{
    controller::Engine,
    http::{self, Server},
    prelude::*,
};

#[derive(Default)]
struct Index;

#[async_trait]
impl Controller for Index {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Response::new().text("Engine"))
    }
}

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    Logger::init();

    let engine = Engine::new(vec![route!("/index" => Index)]);
    Server::new(vec![engine!("/engine" => engine)])
        .launch("0.0.0.0:8000")
        .await
}
