# Rum &dash; Rust Web Framework

Rum is a comprehensive framework for building web applications in Rust. Written using the classic MVC  pattern (model-view-controller), Rum comes standard with everything you need to easily build fast and secure web apps.

### Features overview

- :heavy_check_mark: [HTTP server](examples/quick-start)
- :heavy_check_mark: User-friendly [ORM](examples/orm) to build PostgreSQL queries easily
- :heavy_check_mark: [Dynamic templates](examples/dynamic-templates)
- :heavy_check_mark: [Authentication](examples/auth) & built-in user sessions
- :heavy_check_mark: [Middleware](examples/middleware)
- :heavy_check_mark: [Background jobs](examples/background-jobs) and [scheduled jobs](examples/scheduled-jobs)
- :heavy_check_mark: Database migrations
- :heavy_check_mark: Built-in [RESTful framework](examples/crud) with JSON serialization
- :heavy_check_mark: WebSockets support
- :heavy_check_mark: [Static files](examples/static-files) hosting
- :heavy_check_mark: Tight integration with [Hotwired Turbo](https://turbo.hotwired.dev/) for building [backend-driven SPAs](examples/turbo) 
- :heavy_check_mark: Environment-specific configuration
- :heavy_check_mark: Logging and metrics

## Quick start

To add Rum to your stack, create a Rust binary application and add `rum` and `tokio` to your dependencies:

```bash
cargo add --git https://github.com/levkk/rum rum
cargo add tokio@1 --features full
```

Building an app is then as simple as:

```rust
use rum::prelude::*;
use rum::http::Server;

#[derive(Default)]
struct IndexController;

#[rum::async_trait]
impl Controller for IndexController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::new().html("<h1>Hey Rum!</h1>"))
    }
}

#[tokio::main]
async fn main() {
    Server::new(vec![
        IndexController::default().route("/"),
    ])
    .launch("0.0.0.0:8000")
    .await
    .expect("error shutting down server");
}
```

## Examples

See [examples](examples) for common use cases.

## :construction: Status :construction:

Rum is in early development and not ready for production. Many features including this README are incomplete. Contributions are welcome. Please see [CONTRIBUTING](CONTRIBUTING.md) for guidelines, [ARCHITECTURE](ARCHITECTURE.md) for a tour of the code, and [ROADMAP](ROADMAP.md) for a non-exhaustive list of desired features.

## Documentation

* HTTP Server
* [The ORM](examples/orm/README.md)
* [Dynamic templates](examples/dynamic-templates/README.md)
* [Middleware](examples/middleware/README.md)


## Features

Just like Django or Rails, Rum comes standard with most features needed to build modern web apps. A non-exhaustive list is below, with new features added with every commit.

## HTTP server

Rum has a built-in asynchronous HTTP server which supports millions of concurrent connections.



## Background jobs

Rum comes with its own background jobs queue, workers, and scheduler (also known as clock). The jobs queue is based on Postgres, and uses `SELECT .. FOR UPDATE SKIP LOCKED`, which is an efficient mechanism introduced in recent versions of the database server.

### Creating background jobs

Just like with all previous features, Rum uses the Rust trait system to define background jobs:

```rust
use serde::{Serialize, Deserialize};
use rum::job::{Job, Error as JobError};

#[derive(Clone, Serialize, Deserialize, Default)]
struct SendEmail {
    email: String,
    body: String,
}

#[rum::async_trait]
impl Job for SendEmail {
    async fn execute(&self, args: serde_json::Value) -> Result<(), JobError> {
        // Send an email using Sendgrid or sendmail!
        let args: SendEmail = serde_json::from_value(args)?;
        println!("Sending {} to {}", args.email, args.body);
    }
}
```

Background jobs support arbitrary arguments, which are encoded with JSON, and stored in the database.

### Running jobs

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

### Spawning workers

Workers are processes (Tokio tasks really) that listen for background jobs and execute them. Since we use Tokio, the worker can be launched in the same process as the web server, but doesn't have to be:

```rust
use rum::job::Worker;
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

See the [background jobs](examples/background-jobs) for a complete example.

### Scheduled jobs

Scheduled jobs are background jobs that run automatically based on a schedule, typically defined in the form of a cron:

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

See [scheduled jobs](examples/scheduled-jobs) for a complete example.

#### Cron format

The cron accepts the standard Unix cron format. Up to second precision is allowed (6 stars for every second), with 5 being the minimum (every minute). Non-standard extensions, like `@yearly` are not currently supported, but a PR is welcome.

#### Clock ticks

The scheduler runs every second. If a job is available, it will execute it and immediately (without waiting for the next tick) fetch the next available job from then queue. If no more jobs are available, the scheduler will go back to polling the queue once a second.

#### Durability

Since Rum uses Postgres to store jobs, the job queue is durable &dash; it does not lose jobs &dash; and saves the results of all job runs to a table, which comes in handy when some job does something you didn't expect.

### Spawning multiple workers

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

### Queue guarantees

The Rum job queue has at-least once execution guarantee. This means the queue will attempt to run all jobs at least one time. Since we are using Postgres, jobs do not get lost. That being said, there is no guarantee of a job running more than once, so make sure to write jobs that are idempotent by design &dash; if a job runs more than once, the end result should be the same.

## Database migrations

Rum has built-in migrations for managing the schema of your database in a controlled manner. Migrations are applied sequentially, and each migration is executed inside a transaction for atomicity.

### Writing migrations

Currently Rum doesn't have a CLI (yet) to generate migrations, but creating one is easy. Migrations are SQL files which contain queries. To add a migration, create the a folder called `migrations` and place in it two files:

- the "up" migration
- the "down" migration

The up migration makes the desired changes to your schema, while the down migration reverts those changes. All migrations should be revertible, in case of a problem.

#### Naming convention

Both the up and down migration files should follow this naming convention:

```
VERSION_NAME.(up|down).sql
```

where `VERSION` is any number, `NAME` is the name of the migration (underscores and hyphens allowed), and `(up|down)` is the type of the migration (up or down).

For example, a migration to add the users table could be named `1_users_model.up.sql` while the migration to revert it would be `1_users_model.down.sql`. The `VERSION` number should be unique. Migrations are sorted by `VERSION` before being executed, so all your migrations should be versioned in ascending order of some integer. The current time in seconds is a great choice (`date +%s` in your terminal).

### Running migrations

In your Cargo project, you can create a binary target, e.g. `src/bin/migrations/main.rs` with:

```rust
use rum::prelude::*;

#[tokio::main]
async fn main() {
    Logger::init();

    Migrations::migrate()
        .await
        .expect("migrations failed");
}
```

and execute it, for example:

```
cargo run --bin migrations
```

See the [ORM example](examples/orm) for a complete example.

## RESTful framework

Rum comes with a REST framework (just like Django REST Framework) built-in. Serialization is automatically done with JSON (using `serde_json`) and the API follows the standard CRUD (create, read, update, destroy) pattern.

### Adding REST controllers

To add a REST controller to your Rum app, your controller needs to implement the `ModelController` trait:

```rust
#[derive(rum::macros::ModelController)]
struct UsersController;

#[async_trait]
impl ModelController for UsersController {
    type Model = User;
}
```

The model needs to be serializable into and from JSON, so make sure to derive the appropriate serde traits:

```rust
use serde::{Serialize, Deserialize};

#[derive(Clone, rust::macros::Model, Serialize, Deserialize)]
struct User {
    // Hide this field entirely from the API.
    #[serde(skip_deserializing)]
    id: Option<i64>,

    // The only required field at the API.
    email: String,

    #[serde(with = "time::serde::iso8601", default = "OffsetDateTime::now_utc")]
    created_at: OffsetDateTime,

    #[serde(default="bool::default")]
    admin: bool,
}
```

Adding the controller to the server is then simple:

```rust
#[tokio::main]
async fn main() {
    Server::new(vec![
        UsersController::default().crud("/api/users"),
    ])
    .launch("0.0.0.0:8000")
    .expect("failed to shut down server");
}
```

The `crud` method will automatically implement the following:

| Path | Method | Description |
|------|--------|-------------|
| `/api/users` | GET | List all users. Supports pagination, e.g. `?page_size=25&page=1`. Default page size is 25.|
| `/api/users/:id` | GET | Fetch a user by primary key. |
| `/api/users`| POST | Create a new user. All fields not marked optional or not having serde-specified defaults are required. |
| `/api/users/:id` | PUT | Update a user. Same requirement for fields as the create method above. |
| `/api/users/:id` | PATCH | Update a user. Only the fields that have changed can be supplied. |


You can test this with cURL:

```
$ curl localhost:8000/api/users -d '{"email": "test@test.com"}' -w '\n'

{"email":"test@test.com","created_at":"+002024-10-09T22:59:10.693321000Z","admin":false}
```

### Customizing serialization

Serde allows full control over how fields are serialized and deserialized, including rewriting, renaming, and skipping fields entirely. See [Serde documentation](https://serde.rs/field-attrs.html) for more details.

### Writing your own REST controller

You can write your own REST controller by implementing the `RestController` trait. See [the code](rum/src/controller/mod.rs) for details.

## WebSockets

Rum supports WebSockets out of the box. A WebSockets controller is just another controller which implements the `WebsocketController` trait. 

## Configuration

Configuring Rum apps can be done via environment variables or a TOML configuration file.

### `Rum.toml`

Rum.toml is a configuration file using the TOML configuration language.

#### Example

```toml
[general]
log_queries = true
cache_templates = false
```