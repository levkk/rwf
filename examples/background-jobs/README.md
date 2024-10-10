
# Background jobs

Rwf comes with its own background jobs queue, workers, and scheduler (also known as clock). The jobs queue is based on Postgres, and uses `SELECT .. FOR UPDATE SKIP LOCKED`, which is an efficient mechanism introduced in recent versions of the database server.

## Creating background jobs

Just like with all previous features, Rwf uses the Rust trait system to define background jobs:

```rust
use serde::{Serialize, Deserialize};
use rwf::job::{Job, Error as JobError};

#[derive(Clone, Serialize, Deserialize, Default)]
struct SendEmail {
    email: String,
    body: String,
}

#[rwf::async_trait]
impl Job for SendEmail {
    async fn execute(&self, args: serde_json::Value) -> Result<(), JobError> {
        // Send an email using Sendgrid or sendmail!
        let args: SendEmail = serde_json::from_value(args)?;
        println!("Sending {} to {}", args.email, args.body);
    }
}
```

Background jobs support arbitrary arguments, which are encoded with JSON, and stored in the database.

## Running jobs

Running a job is as simple as scheduling it asynchronously with:

```rust
let job = SendEmail {
    email: "test@hello.com".into(),
    body: "How are you today?".into(),
};

job
    .execute_async(serde_json::to_value(&job)?)
    .await?;
```

## Spawning workers

Workers are processes (Tokio tasks really) that listen for background jobs and execute them. Since we use Tokio, the worker can be launched in the same process as the web server, but doesn't have to be:

```rust
use rwf::job::Worker;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), JobError> {
    Worker::new(vec![
        SendEmail::default().job(),
    ])
    .start()
    .await?;

    sleep(Duration::MAX).await;
}
```

## Durability

Since Rwf uses Postgres to store jobs, the job queue is durable &dash; it does not lose jobs &dash; and saves the results of all job runs to a table, which comes in handy when some job does something you didn't expect.

## Spawning multiple workers

You can spawn as many workers as you think is reasonable for your application. Concurrency is controlled via Postgres, so a particular job won
t run on more than one worker at a time.

To spawn multiple workers inside the same Rust process, call `spawn()` after calling `start()`, for example:

```rust
Worker::new(vec![])
    .start()
    .await?
    .spawn()
    .spawn()
    .spawn();
```

will spawn 4 worker instances. Each instance will run in its own Tokio task.

## Queue guarantees

The Rwf job queue has at-least once execution guarantee. This means the queue will attempt to run all jobs at least one time. Since we are using Postgres, jobs do not get lost. That being said, there is no guarantee of a job running more than once, so make sure to write jobs that are idempotent by design &dash; if a job runs more than once, the end result should be the same.