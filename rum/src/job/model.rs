use crate::job::Error;
use crate::model::{FromRow, Model, ToValue, Value};
use time::OffsetDateTime;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

#[derive(Clone, Debug)]
pub struct JobModel {
    id: Option<i64>,
    name: String,
    args: serde_json::Value,
    created_at: OffsetDateTime,
    started_at: OffsetDateTime,
    completed_at: OffsetDateTime,
    error: Option<String>,
}

impl FromRow for JobModel {
    fn from_row(row: tokio_postgres::Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            args: row.get("args"),
            created_at: row.get("created_at"),
            started_at: row.get("started_at"),
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
            "started_at",
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
            self.started_at.to_value(),
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

    async fn execute<'a>(&self, args: serde_json::Value) -> Result<(), Error>;
    async fn execute_async(&self) -> Result<(), Error> {
        todo!()
    }

    async fn execute_internal(&self) -> Result<(), Error> {
        todo!()
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
    pub fn new(jobs: Vec<impl Job + 'static>) -> Self {
        let jobs = jobs
            .into_iter()
            .map(|job| (job.job_name().to_string(), JobHandler::new(job)))
            .collect();

        Self {
            jobs: Arc::new(jobs),
        }
    }

    pub async fn run(&self) {
        let worker = self.clone();
        let handle = tokio::spawn(async move {
            worker;
        });
    }

    pub fn spawn(&self) {
        let worker = self.clone();
        tokio::spawn(async move {
            worker.run().await;
        });
    }
}
