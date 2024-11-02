use rwf::job::JobModel;
use rwf::prelude::*;
use rwf::serde::Serialize;

pub mod models;

#[derive(Default)]
pub struct Index;

#[async_trait]
impl Controller for Index {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Response::new().redirect("jobs"))
    }
}

#[derive(Default)]
pub struct Jobs;

#[derive(macros::Context)]
struct JobsContext {
    queued: i64,
    running: i64,
    errors: i64,
    latency: i64,
    jobs: Vec<JobModel>,
}

impl JobsContext {
    pub async fn load() -> Result<Self, Error> {
        let mut conn = Pool::connection().await?;
        let queued = JobModel::queued().count(&mut conn).await?;
        let errors = JobModel::errors().count(&mut conn).await?;
        let running = JobModel::running().count(&mut conn).await?;

        let jobs = JobModel::all()
            .order(("id", "DESC"))
            .limit(25)
            .fetch_all(&mut conn)
            .await?;

        let latency = JobModel::queued()
            .order("created_at")
            .take_one()
            .fetch_optional(&mut conn)
            .await?;

        let latency = if let Some(latency) = latency {
            (OffsetDateTime::now_utc() - latency.created_at).whole_seconds()
        } else {
            Duration::seconds(0).whole_seconds()
        };

        Ok(Self {
            queued,
            errors,
            running,
            jobs,
            latency,
        })
    }
}

#[async_trait]
impl Controller for Jobs {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        let template = Template::load("templates/rwf_admin/jobs.html")?;
        Ok(Response::new().html(template.render(JobsContext::load().await?)?))
    }
}

#[derive(Clone, macros::Model, Serialize)]
struct RequestByCode {
    count: i64,
    code: String,
    #[serde(with = "time::serde::rfc2822")]
    created_at: OffsetDateTime,
}

impl RequestByCode {
    fn count(minutes: i64) -> Scope<Self> {
        Self::find_by_sql(
            "WITH timestamps AS (
                SELECT date_trunc('minute', now() - (n || ' minute')::interval) AS created_at FROM generate_series(0, $1::bigint) n
            )
            SELECT
                'ok' AS code,
                COALESCE(e2.count, 0) AS count,
                timestamps.created_at AS created_at
            FROM timestamps
            LEFT JOIN LATERAL (
                SELECT
                    COUNT(*) AS count,
                    DATE_TRUNC('minute', created_at) AS created_at
                FROM rwf_requests
                WHERE
                    created_at BETWEEN timestamps.created_at AND timestamps.created_at + INTERVAL '1 minute'
                    AND code BETWEEN 100 AND 299
                GROUP BY 2
            ) e2 ON true
            UNION ALL
            SELECT
                'warn' AS code,
                COALESCE(e2.count, 0) AS count,
                timestamps.created_at AS created_at
            FROM timestamps
            LEFT JOIN LATERAL (
                SELECT
                    COUNT(*) AS count,
                    DATE_TRUNC('minute', created_at) AS created_at
                FROM rwf_requests
                WHERE
                    created_at BETWEEN timestamps.created_at AND timestamps.created_at + INTERVAL '1 minute'
                    AND code BETWEEN 300 AND 499
                GROUP BY 2
            ) e2 ON true
            UNION ALL
            SELECT
                'error' AS code,
                COALESCE(e2.count, 0) AS coount,
                timestamps.created_at AS created_at
            FROM timestamps
            LEFT JOIN LATERAL (
                SELECT
                    COUNT(*) AS count,
                    DATE_TRUNC('minute', created_at) AS created_at
                FROM rwf_requests
                WHERE
                    created_at BETWEEN timestamps.created_at AND timestamps.created_at + INTERVAL '1 minute'
                    AND code BETWEEN 500 AND 599
                GROUP BY 2
            ) e2 ON true
            ORDER BY 3;",
            &[minutes.to_value()],
        )
    }
}

#[derive(Default)]
pub struct Requests;

#[async_trait]
impl Controller for Requests {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        let requests = {
            let mut conn = Pool::connection().await?;
            RequestByCode::count(60).fetch_all(&mut conn).await?
        };
        let requests = serde_json::to_string(&requests)?;

        render!("templates/rwf_admin/requests.html", "requests" => requests)
    }
}
