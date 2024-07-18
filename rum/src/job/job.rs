use super::Error;
use crate::model::{FromRow, Model, ToValue, Value};
use std::future::Future;

use serde::{de::DeserializeOwned, Serialize};
use time::OffsetDateTime;
use tokio_postgres::{types::Json, Client};

pub trait Job: Serialize + DeserializeOwned + Send + 'static {
    fn execute(&self) -> impl Future<Output = Result<(), Error>> + Send;

    fn async_job_name() -> String {
        std::any::type_name::<Self>().to_string()
    }

    fn execute_async(&self, conn: &Client) -> impl Future<Output = Result<(), Error>> {
        async move {
            let payload = serde_json::to_value(&self)?;

            let model = JobModel {
                id: None,
                name: std::any::type_name::<Self>().to_string(),
                payload,
                created_at: OffsetDateTime::now_utc(),
                executed_at: None,
                completed_at: None,
                error: None,
            };

            let model = model.save().fetch(conn).await?;

            conn.execute(&format!(r#"NOTIFY "jobs", '{}'"#, model.id().unwrap()), &[])
                .await?;

            Ok(())
        }
    }

    fn execute_internal(id: i64, payload: serde_json::Value) {
        let job: Self = serde_json::from_value(payload).expect("deserialization");

        tokio::spawn(async move {
            match job.execute().await {
                Ok(()) => {
                    println!("todo, save job state");
                }

                Err(err) => {
                    println!("todo save error state");
                }
            }
        });
    }
}

#[derive(Clone, Debug)]
pub struct JobModel {
    pub id: Option<i64>,
    pub name: String,
    pub payload: serde_json::Value,
    pub created_at: OffsetDateTime,
    pub executed_at: Option<OffsetDateTime>,
    pub completed_at: Option<OffsetDateTime>,
    pub error: Option<String>,
}

impl JobModel {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn payload(&self) -> serde_json::Value {
        self.payload.clone()
    }
}

impl FromRow for JobModel {
    fn from_row(row: tokio_postgres::Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            payload: row.get("payload"),
            created_at: row.get("created_at"),
            executed_at: row.get("executed_at"),
            completed_at: row.get("completed_at"),
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
            self.name.to_value(),
            self.payload.to_value(),
            self.created_at.to_value(),
            self.executed_at.to_value(),
            self.completed_at.to_value(),
            self.error.to_value(),
        ]
    }

    fn id(&self) -> Option<i64> {
        self.id
    }

    fn column_names() -> Vec<String> {
        vec![
            "name",
            "payload",
            "created_at",
            "executed_at",
            "completed_at",
            "error",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    }
}

#[cfg(test)]
mod test {
    use super::super::Worker;
    use super::*;
    use crate::{logging, model::Pool};
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Debug)]
    struct MyJob {
        user_id: i64,
        order_id: i64,
    }

    impl Job for MyJob {
        fn execute(&self) -> impl Future<Output = Result<(), Error>> {
            println!("executing job: {:?}", self);
            async move { Ok(()) }
        }
    }

    #[tokio::test]
    async fn test_impl_job() {
        logging::configure();
        let mut worker = Worker::default();
        let pool = Pool::new_local();
        let conn = pool.begin().await.expect("transaction");

        conn.execute(
            "
            CREATE TABLE jobs (
                id BIGSERIAL PRIMARY KEY,
                name VARCHAR NOT NULL,
                payload JSONB NOT NULL DEFAULT '{}'::jsonb,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                executed_at TIMESTAMPTZ,
                completed_at TIMESTAMPTZ,
                error VARCHAR
            );
        ",
            &[],
        )
        .await
        .expect("create table");

        MyJob {
            user_id: 5,
            order_id: 10,
        }
        .execute_async(&conn)
        .await
        .expect("execute job");

        worker.add(MyJob::async_job_name().as_str(), MyJob::execute_internal);
        worker.run_once(&conn).await.expect("run once");

        let jobs = JobModel::all()
            .fetch_all(&conn)
            .await
            .expect("find all jobs");
        assert_eq!(jobs.len(), 1);

        println!("{:?}", jobs);
    }
}
