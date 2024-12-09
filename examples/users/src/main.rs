use rwf::controller::{LoginController, LogoutController, SignupController};
use rwf::{http::Server, prelude::*};

mod controllers;
mod models;

#[tokio::main]
async fn main() {
    Logger::init();
    rwf_auth::migrate().await.expect("rwf-auth migrations");

    let signup: SignupController<models::User> =
        SignupController::new("templates/signup.html").redirect("/profile");

    let login: LoginController<models::User> =
        LoginController::new("templates/login.html").redirect("/profile");

    Server::new(vec![
        route!("/signup" => { signup }),
        route!("/login" => { login }),
        route!("/logout" => { LogoutController::default().redirect("/signup") }),
        route!("/profile" => controllers::profile),
    ])
    .launch()
    .await
    .unwrap();
}
