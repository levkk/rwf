use rwf::{http::Server, prelude::*};

mod controllers;
mod models;

#[tokio::main]
async fn main() {
    Logger::init();

    Server::new(vec![
        route!("/signup" => controllers::Signup),
        route!("/login" => controllers::login),
        route!("/profile" => controllers::profile),
    ])
    .launch()
    .await
    .unwrap();
}
