use rwf::prelude::*;
use std::env::args;

#[tokio::main]
async fn main() {
    Logger::init();

    let mut args = args();

    if let Some(direction) = args.nth(1) {
        match direction.as_str() {
            "flush" => Migrations::flush().await.expect("flush failed"),
            _ => Migrations::migrate().await.expect("migrations failed"),
        };
    } else {
        Migrations::migrate().await.expect("migrations failed");
    }
}
