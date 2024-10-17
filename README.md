# Rwf &dash; Rust Web Framework

Rwf is a comprehensive framework for building web applications in Rust. Written using the classic MVC  pattern (model-view-controller), Rwf comes standard with everything you need to easily build fast and secure web apps.

### Documentation

:blue_book: The documentation (work in progress) **[is available here](https://levkk.github.io/rwf/)**.

### Features overview

- :heavy_check_mark: [HTTP server](examples/quick-start)
- :heavy_check_mark: User-friendly [ORM](examples/orm) to build PostgreSQL queries easily
- :heavy_check_mark: [Dynamic templates](examples/dynamic-templates)
- :heavy_check_mark: [Authentication](examples/auth) & built-in user sessions
- :heavy_check_mark: [Middleware](examples/middleware)
- :heavy_check_mark: [Background jobs](examples/background-jobs) and [scheduled jobs](examples/scheduled-jobs)
- :heavy_check_mark: Database migrations
- :heavy_check_mark: Built-in [REST framework](examples/rest) with JSON serialization
- :heavy_check_mark: WebSockets support
- :heavy_check_mark: [Static files](examples/static-files) hosting
- :heavy_check_mark: Tight integration with [Hotwired Turbo](https://turbo.hotwired.dev/) for building [backend-driven SPAs](examples/turbo)
- :heavy_check_mark: Environment-specific configuration
- :heavy_check_mark: Logging and metrics
- :heavy_check_mark: [CLI](rwf-cli)

## Quick start

To add Rwf to your stack, create a Rust binary application and add `rwf` and `tokio` to your dependencies:

```bash
cargo add rwf
cargo add tokio@1 --features full
```

Building an app is then as simple as:

```rust
use rwf::prelude::*;
use rwf::http::Server;

#[derive(Default)]
struct IndexController;

#[async_trait]
impl Controller for IndexController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::new().html("<h1>Hey Rwf!</h1>"))
    }
}

#[tokio::main]
async fn main() {
    Server::new(vec![
        route!("/" => IndexController),
    ])
    .launch("0.0.0.0:8000")
    .await
    .unwrap();
}
```

## Examples

See [examples](examples) for common use cases.

## Documentation

* [ORM & database migrations](examples/orm/README.md)
* [Dynamic templates](examples/dynamic-templates/README.md)
* [Authentication & sessions](examples/auth/README.md)
* [Middleware](examples/middleware/README.md)
* [Background jobs](examples/background-jobs/README.md)
  * [Scheduled jobs](examples/scheduled-jobs/README.md)
* [REST Framework](examples/rest/README.md)

## :construction: Status :construction:

Rwf is in early development and not ready for production. Many features and documentation are incomplete. Contributions are welcome. Please see [CONTRIBUTING](CONTRIBUTING.md) for guidelines, [ARCHITECTURE](ARCHITECTURE.md) for a tour of the code, and [ROADMAP](ROADMAP.md) for a non-exhaustive list of desired features.
