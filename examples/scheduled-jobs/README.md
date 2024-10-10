
# Scheduled jobs

Scheduled jobs are [background jobs](../background-jobs) that run automatically based on a schedule, typically defined in the form of a cron:

```rust
let daily_email = SendEmail {
    email: "boss@hello.com".into(),
    body: "I'm running 5 minutes late, the bus is delayed again.".into(),
};

let scheduled_job = SendEmail::default()
    .schedule(
        serde_json::to_value(&daily_email)?,
        "0 0 9 * * *",
    );

Worker::new(vec![SendEmail::default().job(),])
    .clock(vec![scheduled_job,])
    .start()
    .await?;
```

## Cron format

The cron accepts the standard Unix cron format. Up to second precision is allowed (6 stars for every second), with 5 being the minimum (every minute). Non-standard extensions, like `@yearly` are not currently supported, but a PR is welcome.

## Clock ticks

The scheduler runs every second. If a job is available, it will execute it and immediately (without waiting for the next tick) fetch the next available job from then queue. If no more jobs are available, the scheduler will go back to polling the queue once a second.

## Timezone

The clock runs on the UTC timezone (+00:00 / GMT).