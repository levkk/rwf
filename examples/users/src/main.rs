use rwf::controller::LoginController;
use rwf::{http::Server, prelude::*};

mod controllers;
mod models;

#[tokio::main]
async fn main() {
    Logger::init();

    let signup: LoginController<models::User> =
        LoginController::new("templates/signup.html").redirect("/profile");

    Server::new(vec![
        route!("/signup" => { signup }),
        route!("/login" => controllers::login),
        route!("/profile" => controllers::profile),
    ])
    .launch()
    .await
    .unwrap();
}
