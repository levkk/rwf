use super::{Error, JobHandler, JobModel};

use crate::colors::MaybeColorize;
use time::OffsetDateTime;

use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

use crate::model::{get_connection, get_pool, Model};

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

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

    pub async fn start(self) -> Result<Self, Error> {
        let mut conn = get_connection().await?;
        JobModel::reschedule().execute(&mut conn).await?;

        // Spawn a single instance of the worker.
        self.spawn();

        Ok(self)
    }

    pub async fn run(&self) {
        info!("Background jobs worker started");

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

                        // Run the job in a separate task. If the job panics,
                        // we won't crash this task.
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

                            result => {
                                let err = match result {
                                    Ok(Err(err)) => err.to_string(),
                                    Err(_) => "job panicked".to_string(),
                                    Ok(Ok(_)) => unreachable!(), // Captured above.
                                };

                                error!(
                                    "{} job error ({:.3} ms): {}",
                                    job.name.green(),
                                    elapsed.as_secs_f64() * 1000.0,
                                    err
                                );

                                // Retry with exponential back-off.
                                let delay =
                                    Duration::from_secs(2_i64.pow(job.attempts as u32) as u64);

                                job.error = Some(err);
                                job.attempts += 1;
                                job.start_after = job.created_at + delay;
                                job.started_at = None;

                                job.save().execute(&mut conn).await?;
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
