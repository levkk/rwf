# Rwf &dash; Rust Web Framework

[![Documentation](https://img.shields.io/badge/documentation-blue?style=flat)](https://levkk.github.io/rwf/)
[![Latest crate](https://img.shields.io/crates/v/rwf.svg)](https://crates.io/crates/rwf)
[![Reference docs](https://img.shields.io/docsrs/rwf)](https://docs.rs/rwf/latest/rwf/)


Rwf is a comprehensive framework for building web applications in Rust. Written using the classic MVC  pattern (model-view-controller), Rwf comes standard with everything you need to easily build fast and secure web apps.

## Documentation

&#128216; The documentation **[is available here](https://levkk.github.io/rwf/)**.

## Features overview

- &#10004; [HTTP server](https://github.com/levkk/rwf/tree/main/examples/quick-start)
- &#10004; User-friendly [ORM](https://github.com/levkk/rwf/tree/main/examples/orm) to build PostgreSQL queries easily
- &#10004; [Dynamic templates](https://github.com/levkk/rwf/tree/main/examples/dynamic-templates)
- &#10004; [Authentication](https://github.com/levkk/rwf/tree/main/examples/auth) & built-in user sessions
- &#10004; [Middleware](https://github.com/levkk/rwf/tree/main/examples/middleware)
- &#10004; [Background jobs](https://github.com/levkk/rwf/tree/main/examples/background-jobs) and [scheduled jobs](https://github.com/levkk/rwf/tree/main/examples/scheduled-jobs)
- &#10004; Database migrations
- &#10004; Built-in [REST framework](https://github.com/levkk/rwf/tree/main/examples/rest) with JSON serialization
- &#10004; WebSockets support
- &#10004; [Static files](https://github.com/levkk/rwf/tree/main/examples/static-files) hosting
- &#10004; Tight integration with [Hotwired Turbo](https://turbo.hotwired.dev/) for building [backend-driven SPAs](https://github.com/levkk/rwf/tree/main/examples/turbo)
- &#10004; Environment-specific configuration
- &#10004; Logging and metrics
- &#10004; [CLI](https://github.com/levkk/rwf/tree/main/rwf-cli)
- &#10004; WSGI server for [migrating](https://github.com/levkk/rwf/tree/main/examples/django) from Django/Flask apps
- &#10004; Rack server for [migrating](https://github.com/levkk/rwf/tree/main/examples/rails) from Rails

## Quick start

To add Rwf to your stack, create a Rust binary application and add `rwf` to your dependencies:

```bash
cargo add rwf
```

Building an app is then as simple as:

```rust
use rwf::prelude::*;
use rwf::http::Server;

#[controller]
async fn index() -> Response {
    Response::new().html("<h1>Welcome to Rwf!</h1>")
}

#[tokio::main]
async fn main() {
    Server::new(vec![
        route!("/" => index),
    ])
    .launch()
    .await
    .unwrap();
}
```

## Examples

See [examples](https://github.com/levkk/rwf/tree/main/examples) for common use cases.

## &#128679; Status &#128679;

Rwf is in early development and not ready for production. Many features and documentation are incomplete. Contributions are welcome. Please see [CONTRIBUTING](https://github.com/levkk/rwf/tree/main/CONTRIBUTING.md) for guidelines, [ARCHITECTURE](https://github.com/levkk/rwf/tree/main/ARCHITECTURE.md) for a tour of the code, and [ROADMAP](https://github.com/levkk/rwf/tree/main/ROADMAP.md) for a non-exhaustive list of desired features.
