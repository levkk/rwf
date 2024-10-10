use rum::http::Server;
use rum::prelude::*;

use rand::Rng;

#[derive(Clone, rum::macros::Model)]
struct User {
    id: Option<i64>,
    email: String,
}

#[derive(Default)]
struct IndexController;

#[derive(rum::macros::Context)]
struct IndexTemplate {
    title: String,
    items: Vec<String>,
    show: bool,
    planets: i64,
    users: Vec<User>,
}

#[async_trait]
impl Controller for IndexController {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        let context = IndexTemplate {
            title: "Rum templates are fun!".into(),
            items: vec!["why".into(), "are".into(), "you".into(), "yelling".into()],
            show: rand::thread_rng().gen::<bool>(),
            planets: rand::thread_rng().gen_range(1..=3),
            users: vec![User {
                id: Some(1),
                email: "hello@test.com".into(),
            }],
        };

        let rendered = Template::load("templates/index.html")
            .await?
            .render(&context.try_into()?)?;

        Ok(Response::new().html(rendered))
    }
}

#[tokio::main]
async fn main() {
    Logger::init();

    Server::new(vec![IndexController::default().route("/")])
        .launch("0.0.0.0:8000")
        .await
        .expect("error shutting down server");
}
