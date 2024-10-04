use rum::http::Server;
use rum::logging::setup_logging;
use rum::prelude::*;
use rum::view::prelude::*;

#[derive(Default)]
struct IndexController;

#[derive(rum::macros::Context)]
struct IndexTemplate {
    title: String,
    items: Vec<String>,
}

#[async_trait]
impl Controller for IndexController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let context = IndexTemplate {
            title: "Rum templates are fun!".into(),
            items: vec![
                "why".into(),
                "are".into(),
                "you".into(),
                "yelling".into(),
            ],
        };

        let rendered = Template::cached("templates/index.html")
                .await?
                .render(&context.try_into()?)?;

        Ok(Response::new().html(rendered))
    }
}

#[tokio::main]
async fn main() {
    setup_logging();

    Server::new(vec![IndexController::default().route("/")])
        .launch("0.0.0.0:8000")
        .await
        .expect("error shutting down server");
}
