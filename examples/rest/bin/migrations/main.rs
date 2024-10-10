use rwf::logging::Logger;
use rwf::model::{Error, Migrations};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Enable logging.
    Logger::init();

    // Run migrations.
    Migrations::migrate().await?;

    Ok(())
}
