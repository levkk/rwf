# Job queue guarantees

The background queue is stored in the database, so jobs will not get lost. Workers will attempt to run a job at least once. Even if workers crash, when they are restarted, any running jobs will be rescheduled.

Because of this guarantee, jobs should strive to be idempotent: the same job can be executed multiple times.

## Performance

The job queue is using PostgreSQL's `FOR UPDATE SKIP LOCKED` mechanism, which has been shown to support high concurrency job queues.

## Polling

Workers poll the queue every second. If there are no jobs, the worker goes to sleep and polls again in one second. If a job is available, it will be executed immediately. Once the job completes, the worker will attempt to fetch the next job immediately, restarting this cycle.

## Concurrency

By default, a worker executes one job at a time. This allows to control for background concurrency easily, without complex throttling mechanisms. If you want to execute many jobs concurrently, you can spawn as many workers as you wish. Each worker will poll the queue for jobs once a second.

To spawn more workers, call [`Worker::spawn`](https://docs.rs/rwf/latest/rwf/job/worker/struct.Worker.html#method.spawn) as many times as you wish to have workers, for example:

```rust
let worker = Worker::new(vec![])
  .start()
  .await?
  .spawn()
  .spawn()
  .spawn();
```

The above code will spawn 4 workers in total.
