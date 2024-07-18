use super::{Error, Job, JobModel};
use crate::model::{Model, Pool};
use std::collections::HashMap;
use std::future::Future;

use colored::Colorize;
use time::OffsetDateTime;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

#[derive(Default)]
pub struct Worker {
    jobs: HashMap<String, fn(i64, serde_json::Value)>,
}

impl Worker {
    pub fn add(&mut self, name: &str, f: fn(i64, serde_json::Value)) {
        self.jobs.insert(name.to_string(), f);
    }

    pub async fn run_once(&self, conn: &tokio_postgres::Client) -> Result<(), Error> {
        let mut job = JobModel::filter("executed_at", None::<OffsetDateTime>)
            .filter("completed_at", None::<OffsetDateTime>)
            .order("created_at")
            .limit(1)
            .lock()
            .skip_locked()
            .fetch(&conn)
            .await?;

        match self.jobs.get(job.name()) {
            Some(f) => {
                info!("{} running job ({})", job.name().green(), job.id().unwrap());

                job.executed_at = Some(OffsetDateTime::now_utc());
                let job = job.save().fetch(&conn).await?;

                f(job.id().unwrap(), job.payload())
            }
            None => {
                warn!(
                    "{} job ({}) not registered with worker, skipping",
                    job.name(),
                    job.id().unwrap()
                );
            }
        };

        Ok(())
    }
}
