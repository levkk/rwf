use crate::job::JobModel;
use crate::{prelude::*, view};

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

struct JobsContext {
    queued: i64,
    running: i64,
    errors: i64,
    latency: i64,
    jobs: Vec<JobModel>,
}

impl TryInto<view::Context> for JobsContext {
    type Error = view::Error;

    fn try_into(self) -> Result<view::Context, Self::Error> {
        let mut context = view::Context::new();
        context.set("queued", self.queued)?;
        context.set("running", self.running)?;
        context.set("errors", self.errors)?;
        context.set("latency", self.latency)?;
        context.set("jobs", self.jobs)?;
        Ok(context)
    }
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
