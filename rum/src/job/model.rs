use crate::job::Error;
use crate::model::{get_connection, FromRow, Model, Scope, ToValue, Value};
use time::OffsetDateTime;

use async_trait::async_trait;

/// Job entry in the database-backed job queue.
#[derive(Clone, Debug)]
pub struct JobModel {
    pub id: Option<i64>,
    pub name: String,
    pub args: serde_json::Value,
    pub created_at: OffsetDateTime,
    pub start_after: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub attempts: i64,
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

    ///
    pub fn reschedule() -> Scope<Self> {
        Self::filter("completed_at", Value::Null)
            .not("started_at", Value::Null)
            .update_all(&[("started_at", Value::Null)])
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
            .create()
            .execute(&mut conn)
            .await?;
        Ok(())
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
