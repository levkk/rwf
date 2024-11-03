use rwf::job::JobModel;
use rwf::prelude::*;

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
            .order("completed_at DESC NULLS FIRST")
            .order("started_at DESC NULLS LAST")
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
