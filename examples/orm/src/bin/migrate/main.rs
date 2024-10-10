use rwf::prelude::*;

#[tokio::main]
async fn main() {
    Logger::init();

    Migrations::migrate().await.expect("migrations failed");
}
