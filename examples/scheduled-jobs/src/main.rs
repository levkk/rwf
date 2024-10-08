use rum::http::Server;
use rum::job::{Error as JobError, Job, Worker};
use rum::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Default)]
struct MyJob;

#[rum::async_trait]
impl Job for MyJob {
    async fn execute(&self, _args: serde_json::Value) -> Result<(), JobError> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    Logger::init();

    Worker::new(vec![MyJob::default().job()])
        .clock(vec![
            MyJob::default().schedule(serde_json::Value::Null, "*/5 * * * * *")?
        ])
        .start()
        .await?;

    Server::new(vec![]).launch("0.0.0.0:8000").await?;

    Ok(())
}