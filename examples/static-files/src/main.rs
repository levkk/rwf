use rwf::controller::StaticFiles;
use rwf::http::Server;
use rwf::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    Logger::init();

    Server::new(vec![StaticFiles::serve("static")?])
        .launch()
        .await?;

    Ok(())
}
