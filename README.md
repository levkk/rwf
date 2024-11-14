# Rwf &dash; Rust Web Framework

[![Static Badge](https://img.shields.io/badge/documentation-blue?style=flat)](https://levkk.github.io/rwf/)
[![Latest crate](https://img.shields.io/crates/v/rwf.svg)](https://crates.io/crates/rwf)


Rwf is a comprehensive framework for building web applications in Rust. Written using the classic MVC  pattern (model-view-controller), Rwf comes standard with everything you need to easily build fast and secure web apps.

## Documentation

:blue_book: The documentation **[is available here](https://levkk.github.io/rwf/)**.

## Features overview

- [HTTP server](https://github.com/levkk/rwf/tree/main/examples/quick-start)
- User-friendly [ORM](https://github.com/levkk/rwf/tree/main/examples/orm) to build PostgreSQL queries easily
- [Dynamic templates](https://github.com/levkk/rwf/tree/main/examples/dynamic-templates)
- [Authentication](https://github.com/levkk/rwf/tree/main/examples/auth) & built-in user sessions
- [Middleware](https://github.com/levkk/rwf/tree/main/examples/middleware)
- [Background jobs](https://github.com/levkk/rwf/tree/main/examples/background-jobs) and [scheduled jobs](https://github.com/levkk/rwf/tree/main/examples/scheduled-jobs)
- Database migrations
- Built-in [REST framework](https://github.com/levkk/rwf/tree/main/examples/rest) with JSON serialization
- WebSockets support
- [Static files](https://github.com/levkk/rwf/tree/main/examples/static-files) hosting
- Tight integration with [Hotwired Turbo](https://turbo.hotwired.dev/) for building [backend-driven SPAs](https://github.com/levkk/rwf/tree/main/examples/turbo)
- Environment-specific configuration
- Logging and metrics
- [CLI](https://github.com/levkk/rwf/tree/main/rwf-cli)
- WSGI server for [migrating](https://github.com/levkk/rwf/tree/main/examples/django) from Django/Flask apps
- Rack server for [migrating](https://github.com/levkk/rwf/tree/main/examples/rails) from Rails

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

See [examples](https://github.com/levkk/rwf/tree/main/examples) for common use cases.

## :construction: Status :construction:

Rwf is in early development and not ready for production. Many features and documentation are incomplete. Contributions are welcome. Please see [CONTRIBUTING](https://github.com/levkk/rwf/tree/main/CONTRIBUTING.md) for guidelines, [ARCHITECTURE](https://github.com/levkk/rwf/tree/main/ARCHITECTURE.md) for a tour of the code, and [ROADMAP](https://github.com/levkk/rwf/tree/main/ROADMAP.md) for a non-exhaustive list of desired features.
