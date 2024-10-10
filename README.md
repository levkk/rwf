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
* [Authentication & sessions](examples/auth/README.md)
* [Middleware](examples/middleware/README.md)
* [Background jobs](examples/background-jobs/README.md)
  * [Scheduled jobs](examples/scheduled-jobs/README.md)
* [REST Framework](examples/rest/README.md)


## Features

Just like Django or Rails, Rum comes standard with most features needed to build modern web apps. A non-exhaustive list is below, with new features added with every commit.

## HTTP server

Rum has a built-in asynchronous HTTP server which supports millions of concurrent connections.

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