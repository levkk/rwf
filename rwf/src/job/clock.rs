//! Scheduled jobs implementation.
//!
//! This is also known as a cron.
//!
use super::{Cron, Error, Job, JobHandler};
use crate::{
    colors::MaybeColorize,
    model::{ConnectionGuard, Pool},
};

use std::sync::Arc;
use time::OffsetDateTime;

use serde::Serialize;
use std::time::Instant;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

static LOCK: i64 = 4_334_345_490_663;

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

    async fn check_lock(conn: &ConnectionGuard) -> Result<bool, Error> {
        let rows = conn
            .client()
            .query(&format!("SELECT pg_try_advisory_lock({})", LOCK), &[])
            .await?;
        if let Some(row) = rows.get(0) {
            Ok(row.try_get::<_, bool>(0)?)
        } else {
            Ok(false)
        }
    }

    /// Run the clock. This blocks forever.
    pub async fn run(&self) -> Result<(), Error> {
        info!("Clock is waiting for lock");

        let mut lock = Pool::connection().await?;
        lock.leak();

        while !Self::check_lock(&lock).await? {
            sleep(Duration::from_secs(1)).await;
        }

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
            // This will error out if the connection broke and we lost the lock.
            if !Self::check_lock(&lock).await? {
                return Err(Error::CronConnectionError);
            }

            // Clock should strive to run once a second.
            let remaining = Duration::from_secs(1).saturating_sub(start.elapsed());
            sleep(remaining).await;
        }
    }
}
