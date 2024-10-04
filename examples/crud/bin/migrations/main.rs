use rum::logging::setup_logging;
use rum::model::{Error, Migrations};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Enable logging.
    setup_logging();

    // Run migrations.
    Migrations::migrate().await?;

    Ok(())
}
