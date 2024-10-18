# Jobs overview

Background jobs, also known as asynchronous jobs, are code that can run independently of the main HTTP request/response life cycle. Executing code in background jobs allows you to perform useful work without making the client wait for the job to finish. Examples of background jobs are sending emails or talking to third-party APIs.

Rwf has its own background job queue and workers that can perform those jobs.

## Defining jobs

A background job is any Rust struct that implements the [`Job`](https://docs.rs/rwf/latest/rwf/job/model/trait.Job.html) trait. The only trait method the job needs to implement is the [`async fn execute`](https://docs.rs/rwf/latest/rwf/job/model/trait.Job.html#tymethod.execute) method which accepts job arguments encoded with JSON.

For example, if we wanted to send a welcome email to all users that sign up for your web app, we can do so as a background job:

```rust
use rwf::prelude::*;
use rwf::job::{Error as JobError};
use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize)]
struct WelcomeEmail {
    email: String,
    user_name: String,
}

#[async_trait]
impl Job for WelcomeEmail {
    /// Code in this function will be executed in the background.
    async fn execute(&self, args: serde_json::Value) -> Result<(), JobError> {
        let args: WelcomeEmail = serde_json::from_value(args)?;

        // Send the email to the user
        // with the given email address.

        Ok(())
    }
}
```

## Spawning workers

Once we have background jobs, we need to create background workers that will run in separate threads (Tokio tasks, in reality), and execute those jobs as they are sent to the queue. Spawning workers can be done from anywhere in the code, but typically done so from the `main` function:

```rust
use rwf::job::Worker;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create a new worker with 4 threads.
    let worker = Worker::new(vec![
        WelcomeEmail::default().job()
    ])

    worker.start().await?;

    //  Put the main task to sleep indefinitely.
    sleep(Duration::MAX).await;
}
```

### Sharing processes

Workers can be spawned inside the app without having to create a separate binary application. Since most jobs will be running async code, Tokio will effectively load balance foreground (HTTP requests/responses) and background workloads.

To spawn a worker inside the web app, use the code above without the `sleep`. The [`Worker::start`](https://docs.rs/rwf/latest/rwf/job/worker/struct.Worker.html#method.start) method returns almost immediately, since it only spawns a worker on a separate Tokio task.

## Scheduling jobs

With the background jobs defined and the workers running, we can start scheduling jobs to run in the background. A job can be scheduled to run from anywhere in the code by calling the [`Job::execute_async`](https://docs.rs/rwf/latest/rwf/job/model/trait.Job.html#method.execute_async) method:

```rust
let email = WelcomeEmail {
    email: "new-user@example.com".to_string(),
    user_name: "Alice".to_string(),
};

// Convert the job to a JSON value.
let args = serde_json::to_value(&email)?;

// Schedule the job to run in the background
// as soon as possible.
email.execute_async(args).await?;
```

The `execute_async` method creates a record of the job in the queue and returns immediately without doing the actual work. This makes this method very quick so you can schedule multiple jobs inside a controller without it having noticeable effect on endpoint latency.
