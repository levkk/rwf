use super::{Cron, Error, JobHandler};
use time::OffsetDateTime;

pub struct ScheduledJob {
    job: Box<JobHandler>,
    args: serde_json::Value,
    cron: Cron,
}

impl ScheduledJob {
    pub async fn schedule(&self, time: &OffsetDateTime) -> Result<(), Error> {
        self.job.job.execute_async(self.args.clone()).await?;

        Ok(())
    }

    pub fn should_run(&self, time: &OffsetDateTime) -> bool {
        self.cron.should_run(time)
    }
}
