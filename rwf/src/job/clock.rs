//! Cron implementation.
use super::{Cron, Error, Job, JobHandler};
use crate::colors::MaybeColorize;

use std::sync::Arc;
use time::OffsetDateTime;

use serde::Serialize;
use tokio::time::{interval, Duration};
use tracing::{error, info};

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

    /// Run the clock. This is an infinite loop.
    pub async fn run(&self) {
        info!("Clock started");

        let mut clock = interval(Duration::from_secs(1));

        loop {
            clock.tick().await;
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
        }
    }
}
