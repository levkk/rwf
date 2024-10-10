use rwf::controller::StaticFiles;
use rwf::http::Server;
use rwf::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    Logger::init();

    Server::new(vec![StaticFiles::serve("static")?])
        .launch("0.0.0.0:8000")
        .await?;

    Ok(())
}
