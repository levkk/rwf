use rwf::{http::Server, prelude::*};
use rwf_auth::controllers::{LogoutController, PasswordController};

mod controllers;
mod models;

#[tokio::main]
async fn main() {
    Logger::init();

    Server::new(vec![
        route!("/auth" => {
            PasswordController::template("templates/login.html")
                .redirect("/profile")
        }),
        route!("/logout" => { LogoutController::redirect("/") }),
        route!("/profile" => controllers::profile),
    ])
    .launch()
    .await
    .unwrap();
}
