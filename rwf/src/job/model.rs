//! Job model.
//!
//! Used internally, but can be used externally by knowledgeable callers
//! to schedule jobs or fetch statistics about the job queue.
use crate::colors::MaybeColorize;
use crate::job::{clock::ScheduledJob, Error};
use crate::model::{get_connection, FromRow, Model, Scope, ToValue, Value};
use serde::Serialize;
use time::{Duration, OffsetDateTime};

use async_trait::async_trait;
use tracing::info;

/// Job entry in the database-backed job queue.
#[derive(Clone, Debug)]
pub struct JobModel {
    pub id: Option<i64>,
    pub name: String,
    pub args: serde_json::Value,
    pub created_at: OffsetDateTime,
    pub start_after: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub attempts: i32,
    pub retries: i64,
    pub completed_at: Option<OffsetDateTime>,
    pub error: Option<String>,
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

    fn new_with_delay(name: &str, args: serde_json::Value, delay: Duration) -> Self {
        let mut job = Self::new(name, args);
        job.start_after = OffsetDateTime::now_utc() + delay;
        job
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

    /// Fetch jobs that should be rescheduled.
    ///
    /// This happens if a worker crashed.
    pub fn reschedule() -> Scope<Self> {
        Self::filter("completed_at", Value::Null)
            .not("started_at", Value::Null)
            .update_all(&[("started_at", Value::Null)])
    }

    /// Fetch all instances of this job that have been scheduled to run.
    /// and should run now.
    pub fn scheduled(&self) -> Scope<Self> {
        Self::filter("completed_at", Value::Null)
            .filter("start_after", self.start_after)
            .filter("started_at", Value::Null)
            .filter("name", &self.name)
    }

    /// Get all jobs that are currently running.
    pub fn running() -> Scope<Self> {
        Self::filter("completed_at", Value::Null).not("started_at", Value::Null)
    }

    /// Get all jobs that are currently queued.
    pub fn queued() -> Scope<Self> {
        Self::filter("completed_at", Value::Null).filter("started_at", Value::Null)
    }

    /// Get all jobs that had a problem.
    pub fn errors() -> Scope<Self> {
        Self::all()
            .not("completed_at", Value::Null)
            .not("error", Value::Null)
    }
}

impl FromRow for JobModel {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, crate::model::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            args: row.try_get("args")?,
            created_at: row.try_get("created_at")?,
            start_after: row.try_get("start_after")?,
            started_at: row.try_get("started_at")?,
            attempts: row.try_get("attempts")?,
            retries: row.try_get("retries")?,
            completed_at: row.try_get("completed_at")?,
            error: row.try_get("error")?,
        })
    }
}

impl Model for JobModel {
    fn id(&self) -> Value {
        self.id.to_value()
    }

    fn table_name() -> &'static str {
        "rwf_jobs"
    }

    fn primary_key() -> &'static str {
        "id"
    }

    fn foreign_key() -> &'static str {
        "rwf_job_id"
    }

    fn column_names() -> &'static [&'static str] {
        &[
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

/// Asynchronous background job.
///
/// Can execute arbitrary tasks in the background without blocking
/// foreground HTTP requests.
#[async_trait]
pub trait Job: Sync + Send {
    /// Execute the job.
    ///
    /// Implement this method with the code you want to run in the background.
    /// Arguments are passed in using JSON.
    async fn execute(&self, args: serde_json::Value) -> Result<(), Error>;

    /// Schedule this job to run in the background.
    ///
    /// This method schedules the job in the queue and returns immediately without
    /// running the job.
    async fn execute_async(&self, args: serde_json::Value) -> Result<(), Error> {
        let mut conn = get_connection().await?;
        JobModel::new(self.job_name(), args)
            .save()
            .execute(&mut conn)
            .await?;

        info!("job {} scheduled to run now", self.job_name().green());

        Ok(())
    }

    async fn execute_delay(&self, args: serde_json::Value, delay: Duration) -> Result<(), Error> {
        let mut conn = get_connection().await?;
        JobModel::new_with_delay(self.job_name(), args, delay)
            .save()
            .execute(&mut conn)
            .await?;

        info!(
            "job {} scheduled to run in {}s",
            self.job_name().green(),
            delay.whole_seconds()
        );

        Ok(())
    }

    fn schedule(self, args: serde_json::Value, schedule: &str) -> Result<ScheduledJob, Error>
    where
        Self: Sized + 'static,
    {
        ScheduledJob::new(schedule, self, args)
    }

    /// Name of the job. Must be globally unique.
    ///
    /// Currently the type name of the struct is used, so
    /// global uniqueness requirement is satisfied. Be careful
    /// overriding this method.
    fn job_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Wrap the job into a boxed wrapper.
    ///
    /// Do not override this method.
    fn job(self) -> JobHandler
    where
        Self: Sized + 'static,
    {
        JobHandler::new(self)
    }
}

/// Wrapper around the concrete job implementation.
pub struct JobHandler {
    pub job: Box<dyn Job>,
}

impl JobHandler {
    /// Wrap the job and box it.
    pub fn new(job: impl Job + 'static) -> Self {
        Self { job: Box::new(job) }
    }
}

#[inline]
pub async fn queue<T: Job + Serialize>(job: &T) -> Result<(), Error> {
    let args = serde_json::to_value(job)?;
    job.execute_async(args).await
}

#[inline]
pub async fn queue_delay<T: Job + Serialize>(job: &T, delay: Duration) -> Result<(), Error> {
    let args = serde_json::to_value(job)?;
    job.execute_delay(args, delay).await
}

#[inline]
pub async fn queue_async<T: Job + Serialize>(job: &T) -> Result<(), Error> {
    queue(job).await
}
