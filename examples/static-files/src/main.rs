use rum::controller::StaticFiles;
use rum::http::Server;
use rum::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    Logger::init();

    Server::new(vec![StaticFiles::serve("static")?])
        .launch("0.0.0.0:8000")
        .await?;

    Ok(())
}
