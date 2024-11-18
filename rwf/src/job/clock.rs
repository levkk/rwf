//! Scheduled jobs implementation.
//!
//! This is also known as a cron.
//!
use super::{Cron, Error, Job, JobHandler};
use crate::{colors::MaybeColorize, model::Pool};

use std::sync::Arc;
use time::OffsetDateTime;

use serde::Serialize;
use std::time::Instant;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

static LOCK: i64 = 4334345490663;

/// A job that runs on a schedule.
pub struct ScheduledJob {
    job: JobHandler,
    args: serde_json::Value,
    cron: Cron,
}

impl ScheduledJob {
    /// Execute the job.
    pub async fn schedule(&self) -> Result<(), Error> {
        self.job.job.execute_async(self.args.clone()).await?;

        Ok(())
    }

    /// Check if the job should run at the specified time.
    pub fn should_run(&self, time: &OffsetDateTime) -> bool {
        self.cron.should_run(time)
    }

    /// Get the job handler.
    pub fn job(&self) -> &Box<dyn Job> {
        &self.job.job
    }

    /// Create new scheduled job.
    pub fn new(
        schedule: &str,
        job: impl Job + 'static,
        args: impl Serialize,
    ) -> Result<Self, Error> {
        let cron = Cron::parse(schedule)?;
        let handler = JobHandler::new(job);
        let args = serde_json::to_value(args)?;

        Ok(Self {
            job: handler,
            args,
            cron,
        })
    }
}

/// The clock.
#[derive(Clone)]
pub struct Clock {
    jobs: Arc<Vec<ScheduledJob>>,
}

impl Clock {
    /// Create new clock.
    pub fn new(jobs: Vec<ScheduledJob>) -> Self {
        Self {
            jobs: Arc::new(jobs),
        }
    }

    /// Run the clock. This blocks forever.
    pub async fn run(&self) -> Result<(), Error> {
        info!("Clock is waiting for lock");

        let mut lock = Pool::connection().await?;
        lock.leak();

        lock.client()
            .execute(&format!("SELECT pg_advisory_lock({})", LOCK), &[])
            .await?;

        info!("Clock is running");

        loop {
            let start = Instant::now();
            let now = OffsetDateTime::now_utc();
            let jobs = self.jobs.clone();

            tokio::spawn(async move {
                for job in jobs.iter() {
                    if job.should_run(&now) {
                        match job.schedule().await {
                            Ok(_) => (),
                            Err(err) => {
                                error!(
                                    "job {} failed to schedule: {:?}",
                                    job.job().job_name().green(),
                                    err
                                );
                            }
                        }
                    }
                }
            });

            // Make sure we still have a lock.
            lock.query_cached(&format!("SELECT pg_advisory_lock({})", LOCK), &[])
                .await?;

            // Clock should strive to run once a second.
            let remaining = Duration::from_secs(1).saturating_sub(start.elapsed());
            sleep(remaining).await;
        }
    }
}
