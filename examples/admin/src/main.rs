use rwf::admin;
use rwf::{
    controller::TurboStream,
    http::{self, Server},
    prelude::*,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    Logger::init();
    Migrations::migrate().await?;

    #[cfg(debug_assertions)]
    rwf::hmr::hmr(PathBuf::from("templates"));

    let admin = admin::engine();
    Server::new(vec![
        engine!("/admin" => admin),
        route!("/turbo-stream" => TurboStream),
    ])
    .launch("0.0.0.0:8000")
    .await
}
