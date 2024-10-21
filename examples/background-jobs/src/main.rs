use log::info;
use rwf::http::Server;
use rwf::job::{Error as JobError, Worker};
use rwf::model::Migrations;
use rwf::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
struct SendEmailJob {
    email: String,
    body: String,
}

#[rwf::async_trait]
impl Job for SendEmailJob {
    async fn execute(&self, args: serde_json::Value) -> Result<(), JobError> {
        let email: Self = serde_json::from_value(args)?;

        info!("executing {} with args {:?}", self.job_name(), email,);

        Ok(())
    }
}

#[derive(Default)]
struct IndexController;

#[async_trait]
impl Controller for IndexController {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        let job = SendEmailJob {
            email: "test@test.com".into(),
            body: "Hey, this is Rum, how are you?".into(),
        };

        queue_async(&job).await?;

        Ok(Response::new().html("<h1>Job scheduled!</h1>"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    Logger::init();

    Migrations::migrate().await?;

    Worker::new(vec![SendEmailJob::default().job()])
        .start()
        .await?;

    Server::new(vec![IndexController::default().route("/")])
        .launch("0.0.0.0:8000")
        .await?;

    Ok(())
}
