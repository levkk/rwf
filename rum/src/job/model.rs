use crate::job::Error;
use crate::model::{get_connection, get_pool, FromRow, Model, Scope, ToValue, Value};
use time::OffsetDateTime;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use colored::Colorize;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

#[derive(Clone, Debug)]
pub struct JobModel {
    id: Option<i64>,
    name: String,
    args: serde_json::Value,
    created_at: OffsetDateTime,
    start_after: OffsetDateTime,
    started_at: Option<OffsetDateTime>,
    attempts: i64,
    retries: i64,
    completed_at: Option<OffsetDateTime>,
    error: Option<String>,
}

impl JobModel {
    fn new(name: &str, args: serde_json::Value) -> Self {
        Self {
            id: None,
            name: name.to_string(),
            args,
            created_at: OffsetDateTime::now_utc(),
            start_after: OffsetDateTime::now_utc(),
            started_at: None,
            attempts: 0,
            retries: 25,
            completed_at: None,
            error: None,
        }
    }

    /// Fetch the next job from the queue.
    ///
    /// Locks the job from being fetched by other workers.
    pub fn next() -> Scope<Self> {
        Self::filter("completed_at", Value::Null)
            .filter("started_at", Value::Null)
            .filter_lt("attempts", JobModel::column("retries"))
            .filter_lte("start_after", Value::function("NOW")) // use database time
            .order((JobModel::column("created_at"), "ASC"))
            .take_one()
            .lock()
            .skip_locked()
    }
}

impl FromRow for JobModel {
    fn from_row(row: tokio_postgres::Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            args: row.get("args"),
            created_at: row.get("created_at"),
            start_after: row.get("start_after"),
            started_at: row.get("started_at"),
            attempts: row.get("attempts"),
            retries: row.get("retries"),
            completed_at: row.get("completed_at"),
            error: row.get("error"),
        }
    }
}

impl Model for JobModel {
    fn id(&self) -> Value {
        self.id.to_value()
    }

    fn table_name() -> String {
        "rum_jobs".to_string()
    }

    fn primary_key() -> String {
        "id".to_string()
    }

    fn foreign_key() -> String {
        "rum_job_id".to_string()
    }

    fn column_names() -> Vec<String> {
        vec![
            "name",
            "args",
            "created_at",
            "start_after",
            "started_at",
            "attempts",
            "retries",
            "completed_at",
            "error",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn values(&self) -> Vec<Value> {
        vec![
            self.name.to_value(),
            self.args.to_value(),
            self.created_at.to_value(),
            self.start_after.to_value(),
            self.started_at.to_value(),
            self.attempts.to_value(),
            self.retries.to_value(),
            self.completed_at.to_value(),
            self.error.to_value(),
        ]
    }
}

#[async_trait]
pub trait Job: Sync + Send {
    fn job_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn job(self) -> JobHandler
    where
        Self: Sized + 'static,
    {
        JobHandler::new(self)
    }

    async fn execute(&self, args: serde_json::Value) -> Result<(), Error>;
    async fn execute_async(&self, args: serde_json::Value) -> Result<(), Error> {
        let mut conn = get_connection().await?;
        JobModel::new(self.job_name(), args)
            .create()
            .execute(&mut conn)
            .await?;
        Ok(())
    }
}

pub struct JobHandler {
    job: Box<dyn Job>,
}

impl JobHandler {
    pub fn new(job: impl Job + 'static) -> Self {
        Self { job: Box::new(job) }
    }
}

#[derive(Clone)]
pub struct Worker {
    jobs: Arc<HashMap<String, JobHandler>>,
}

impl Worker {
    pub fn new(jobs: Vec<JobHandler>) -> Self {
        let jobs = jobs
            .into_iter()
            .map(|job| (job.job.job_name().to_string(), job))
            .collect();
        Self {
            jobs: Arc::new(jobs),
        }
    }

    pub async fn run(&self) {
        info!("background jobs worker started");

        loop {
            let worker = self.clone();
            let run_result = tokio::spawn(async move {
                let pool = get_pool();

                let job = pool
                    .with_transaction(|mut transaction| async move {
                        let job = JobModel::next().fetch_optional(&mut transaction).await?;

                        let job = if let Some(mut job) = job {
                            job.started_at = Some(OffsetDateTime::now_utc());
                            Ok(Some(job.save().fetch(&mut transaction).await?))
                        } else {
                            Ok(None)
                        };

                        transaction.commit().await?;

                        job
                    })
                    .await?;

                if let Some(mut job) = job {
                    if worker.jobs.get(&job.name).is_some() {
                        let worker = worker.clone();
                        let args = job.args.clone();
                        let name = job.name.clone();
                        let now = Instant::now();

                        let result = tokio::spawn(async move {
                            let registered_job = &worker.jobs[&name];

                            registered_job.job.execute(args).await?;

                            Ok::<(), Error>(())
                        })
                        .await;

                        let elapsed = now.elapsed();

                        let mut conn = get_connection().await?;

                        match result {
                            Ok(Ok(())) => {
                                info!(
                                    "{} job finished ({:.3} ms)",
                                    job.name.green(),
                                    elapsed.as_secs_f64() * 1000.0
                                );
                                job.completed_at = Some(OffsetDateTime::now_utc());
                                job.attempts += 1;
                                job.save().execute(&mut conn).await?;
                            }
                            Ok(Err(err)) => {
                                error!(
                                    "{} job error ({:.3} ms): {:?}",
                                    job.name.green(),
                                    elapsed.as_secs_f64() * 1000.0,
                                    err
                                );

                                // Retry with expoential backoff.
                                let delay =
                                    Duration::from_secs(2_i64.pow(job.attempts as u32) as u64);

                                job.error = Some(err.to_string());
                                job.attempts += 1;
                                job.start_after = job.created_at + delay;
                                job.started_at = None;

                                job.save().execute(&mut conn).await?;
                            }
                            Err(_) => {
                                error!("worker crashed because of panic inside job, this is a bug");
                            }
                        }
                    } else {
                        warn!("worker received unknown job: \"{}\"", job.name);
                    }
                } else {
                    sleep(Duration::from_millis(1000)).await;
                }

                Ok::<(), Error>(())
            })
            .await;

            match run_result {
                Ok(Ok(_)) => (),
                Ok(Err(err)) => {
                    error!("worker crashed with error, restarting: {:?}", err);
                }
                Err(_) => {
                    error!("worker panicked, which is a bug in the worker, restarting");
                    sleep(Duration::from_millis(1000)).await;
                }
            }
        }
    }

    pub fn spawn(&self) -> &Self {
        let worker = self.clone();
        tokio::spawn(async move {
            worker.run().await;
        });
        self
    }
}
