use super::{Error, Worker};
use crate::model::{get_pool, FromRow, Model, ToValue, Value};
use std::future::Future;

use serde::{de::DeserializeOwned, Serialize};
use time::{Duration, OffsetDateTime};
use tokio_postgres::{types::Json, Client};

static MAX_RETRIES: i64 = 25;

pub trait Job: Serialize + DeserializeOwned + Send + 'static {
    fn execute(&self) -> impl Future<Output = Result<(), Error>> + Send;

    fn async_job_name() -> String {
        std::any::type_name::<Self>().to_string()
    }

    fn execute_async(&self, conn: &Client) -> impl Future<Output = Result<(), Error>> {
        async move {
            JobModel::create(
                &Self::async_job_name(),
                serde_json::to_value(self).expect("job serialization error"),
                None,
                conn,
            )
            .await?;

            Ok(())
        }
    }

    fn execute_delay(
        &self,
        conn: &Client,
        delay: Duration,
    ) -> impl Future<Output = Result<(), Error>> {
        async move {
            JobModel::create(
                &Self::async_job_name(),
                serde_json::to_value(self).expect("job serialization error"),
                Some(delay),
                &conn,
            )
            .await?;

            Ok(())
        }
    }

    fn execute_internal(id: i64, payload: serde_json::Value) {
        let job: Self = match serde_json::from_value(payload) {
            Ok(job) => job,
            Err(_) => return,
        };

        tokio::spawn(async move {
            match job.execute().await {
                Ok(()) => {
                    let pool = get_pool();
                    let conn = pool.get().await?;
                    let mut job = JobModel::find(id).fetch(&conn).await?;

                    job.completed_at = Some(OffsetDateTime::now_utc());
                    job.save().execute(&conn).await?;
                }

                Err(err) => {
                    let pool = get_pool();
                    let conn = pool.begin().await?;

                    let mut job = JobModel::find(id).lock().fetch(&conn).await?;

                    job.retries -= 1;
                    job.error = Some(err.to_string());
                    job.start_after = OffsetDateTime::now_utc()
                        + Duration::seconds(2_i32.pow((MAX_RETRIES - job.retries) as u32) as i64); // Exponential back-off

                    job.save().execute(&conn).await?;

                    conn.commit().await?;
                }
            }

            Ok::<(), Error>(())
        });
    }

    fn register(worker: &mut Worker) {
        worker.add(Self::async_job_name().as_str(), Self::execute_internal);
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
    pub start_after: OffsetDateTime,
    pub retries: i64,
    pub error: Option<String>,
}

impl JobModel {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn payload(&self) -> serde_json::Value {
        self.payload.clone()
    }

    async fn create(
        name: &str,
        payload: serde_json::Value,
        delay: Option<Duration>,
        conn: &tokio_postgres::Client,
    ) -> Result<Self, Error> {
        let model = JobModel {
            id: None,
            name: std::any::type_name::<Self>().to_string(),
            payload,
            created_at: OffsetDateTime::now_utc(),
            executed_at: None,
            completed_at: None,
            start_after: OffsetDateTime::now_utc() + delay.unwrap_or(Duration::seconds(0)),
            retries: MAX_RETRIES,
            error: None,
        };

        let model = model.save().fetch(conn).await?;

        conn.execute(&format!(r#"NOTIFY "jobs", '{}'"#, model.id().unwrap()), &[])
            .await?;

        Ok(model)
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
            start_after: row.get("start_after"),
            retries: row.get("retries"),
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
            self.start_after.to_value(),
            self.retries.to_value(),
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
            "start_after",
            "retries",
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
    use once_cell::sync::OnceCell;
    use serde::Deserialize;

    static JOB_RAN: OnceCell<bool> = OnceCell::new();

    #[derive(Serialize, Deserialize, Debug)]
    struct MyJob {
        user_id: i64,
        order_id: i64,
    }

    impl Job for MyJob {
        fn execute(&self) -> impl Future<Output = Result<(), Error>> {
            JOB_RAN.set(true).expect("job ran");
            async move { Ok(()) }
        }
    }

    #[tokio::test]
    async fn test_impl_job() {
        logging::configure();
        let mut worker = Worker::new(&[MyJob::register]);

        let pool = Pool::new_local();

        pool.with_transaction(|transaction| async move {
            transaction.execute("SELECT 1", &[]).await?;
            transaction.commit().await?;
            Ok(())
        })
        .await
        .expect("with transaction");

        let conn = pool.begin().await.expect("transaction");

        conn.execute(
            "
            CREATE TABLE IF NOT EXISTS jobs (
                id BIGSERIAL PRIMARY KEY,
                name VARCHAR NOT NULL,
                payload JSONB NOT NULL DEFAULT '{}'::jsonb,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                executed_at TIMESTAMPTZ,
                completed_at TIMESTAMPTZ,
                start_after TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                retries BIGINT NOT NULL DEFAULT 25,
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

        // conn.commit().await?;

        worker.run_once(&conn).await.expect("run once");

        let jobs = JobModel::all()
            .fetch_all(&conn)
            .await
            .expect("find all jobs");
        assert_eq!(jobs.len(), 1);

        println!("{:?}", jobs);

        assert!(JOB_RAN.get().unwrap());
    }
}
