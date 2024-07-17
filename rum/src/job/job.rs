use super::Error;
use crate::model::{FromRow, Model, ToValue, Value};
use std::future::Future;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio_postgres::{types::Json, Client};

pub trait Job: Serialize {
    fn execute(&self) -> Result<(), Error>;

    fn execute_async(&self, conn: &Client) -> impl Future<Output = Result<(), Error>> {
        async move {
            let payload = serde_json::to_value(&self)?;

            let model = JobModel {
                id: None,
                payload,
                created_at: OffsetDateTime::now_utc(),
                executed_at: None,
                error: None,
            };

            model.save().fetch(conn).await?;

            Ok(())
        }
    }
}

#[derive(Clone, Debug)]
struct JobModel {
    id: Option<i64>,
    payload: serde_json::Value,
    created_at: OffsetDateTime,
    executed_at: Option<OffsetDateTime>,
    error: Option<String>,
}

impl FromRow for JobModel {
    fn from_row(row: tokio_postgres::Row) -> Self {
        Self {
            id: row.get("id"),
            payload: row.get("payload"),
            created_at: row.get("created_at"),
            executed_at: row.get("executed_at"),
            error: row.get("error"),
        }
    }
}

impl Model for JobModel {
    fn table_name() -> String {
        "jobs".to_string()
    }

    fn primary_key() -> String {
        "id".to_string()
    }

    fn foreign_key() -> String {
        "job_id".to_string()
    }

    fn values(&self) -> Vec<Value> {
        vec![
            self.payload.to_value(),
            self.created_at.to_value(),
            self.executed_at.to_value(),
            self.error.to_value(),
        ]
    }

    fn id(&self) -> Option<i64> {
        self.id
    }

    fn column_names() -> Vec<String> {
        vec!["payload", "created_at", "executed_at", "error"]
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }
}
