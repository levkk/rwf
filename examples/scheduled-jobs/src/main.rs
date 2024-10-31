use rwf::job::{Error as JobError, Job, Worker};
use rwf::prelude::*;

use serde::{Deserialize, Serialize};
use tokio::time::sleep;

#[derive(Clone, Serialize, Deserialize, Default)]
struct MyJob;

#[rwf::async_trait]
impl Job for MyJob {
    async fn execute(&self, _args: serde_json::Value) -> Result<(), JobError> {
        sleep(std::time::Duration::from_secs(1)).await;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    Logger::init();

    Migrations::migrate().await?;

    Worker::new(vec![MyJob::default().job()])
        .clock(vec![
            MyJob::default().schedule(serde_json::Value::Null, "*/5 * * * * *")?
        ])
        .start()
        .await?;

    sleep(std::time::Duration::MAX).await;

    Ok(())
}
