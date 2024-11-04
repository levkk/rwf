# Cron jobs

Cron jobs, or scheduled jobs, are background jobs that are performed automatically based on a schedule. For example, if you want to send a newsletter to your users every week, you can create a background job and schedule it to run weekly using the built-in cron.

## Defining scheduled jobs

A scheduled job is a regular [background job](index.md), for example:

```rust
use rwf::prelude::*;
use rwf::job::{Error as JobError};

#[derive(Default, Debug, Serialize, Deserialize)]
struct WeeklyNewsletter;

#[async_trait]
impl Job for WeeklyNewsletter {
    /// Code in this function will be executed in
    /// the background.
    async fn execute(&self, _args: serde_json::Value) -> Result<(), JobError> {
        // Send the newsletter to all users.
        Ok(())
    }
}
```

To run a job on a schedule, you need to add it in two places:

- The list of jobs the worker can run
- The crontab (or the clock, as we call it)

```rust
// Crontab
let schedule = vec![
    WeeklyNewsletter::default()
        .schedule(
            serde_json::Value::Null,
            "0 0 * * 0",
        ), // Every Sunday at midnight
];

// Background jobs
let jobs = vec![
    WeeklyNewsletter::default().job()
];

let worker = Worker::new(jobs)
    .clock(schedule);

worker.start().await?;
```
