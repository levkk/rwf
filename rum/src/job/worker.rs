use super::{Error, Job, JobModel};
use crate::model::{Model, Pool, Value};
use std::collections::HashMap;
use std::future::Future;

use colored::Colorize;
use time::OffsetDateTime;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

#[derive(Default, Clone)]
pub struct Worker {
    jobs: HashMap<String, fn(i64, serde_json::Value)>,
}

impl Worker {
    pub fn add(&mut self, name: &str, f: fn(i64, serde_json::Value)) {
        self.jobs.insert(name.to_string(), f);
    }

    async fn run_job(&self, mut job: JobModel, conn: &tokio_postgres::Client) -> Result<(), Error> {
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

    pub async fn run_once(&self, conn: &tokio_postgres::Client) -> Result<(), Error> {
        let mut job = JobModel::filter("executed_at", Value::Null)
            .filter("completed_at", Value::Null)
            .filter_gte("start_after", OffsetDateTime::now_utc())
            .filter_gt("retries", 0)
            .order("created_at")
            .limit(1)
            .lock()
            .skip_locked()
            .fetch(&conn)
            .await?;

        self.run_job(job, conn).await
    }

    pub async fn run_notified(&self, conn: &tokio_postgres::Client, id: i64) -> Result<(), Error> {
        let mut job = JobModel::find(id).fetch(&conn).await?;

        self.run_job(job, conn).await
    }

    /// Launch an instance of a worker.
    ///
    /// You can launch as many of these as you want. They will run concurrently.
    pub fn run(&self, pool: Pool) {
        let worker = self.clone();
        tokio::spawn(async move {
            loop {
                let transaction = match pool.begin().await {
                    Ok(transaction) => {
                        match worker.run_once(&transaction).await {
                            Ok(()) => (),
                            Err(_) => (),
                        };

                        match transaction.commit().await {
                            Ok(()) => (),
                            Err(_) => (),
                        }
                    }

                    Err(_) => (),
                };

                sleep(Duration::from_secs(1)).await;
            }
        });
    }

    pub async fn spawn_workers(&self, pool: Pool, num_workers: usize) {
        for _ in 0..num_workers {
            self.run(pool.clone());
        }
    }
}
