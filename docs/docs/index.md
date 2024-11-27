# Getting started

Rust Web Framework (Rwf for short) is a framework for building web applications using the Rust programming language. It aims to be comprehensive by providing all necessary features for building modern, fast, and secure web apps.

Rwf has very few dependencies and is easy use with new or existing Rust applications.

## Install Rust

If you haven't already, install the Rust compiler and tools from [rust-lang.org](https://rust-lang.org). Rwf doesn't use any nightly or experimental features,
so the stable version of the compiler will work.

## Create a project

Rwf can be used with any Rust binary or library project. If you don't have one already, you can create it with Cargo[^1]:

```bash
cargo init --bin rwf-web-app &&
cd rwf-web-app
```

[^1]: [Cargo](https://doc.rust-lang.org/cargo/) is the package manager for the Rust programming language. It can install any open source library available on [crates.io](https://crates.io).

## Add Rwf

Rwf primarily consists of the [`rwf`](https://crates.io/crates/rwf) crate[^2]. Add it to your project with Cargo:

```
cargo add rwf
```

[^2]: A crate is a library that can be used in Rust applications. It's similar to packages in JavaScript or Python.

## Build an application

With the [`rwf`](https://crates.io/crates/rwf) crate added, you're ready to build your first web application in Rust.
Rwf is [MVC](https://en.wikipedia.org/wiki/Model%E2%80%93view%E2%80%93controller) (model-view-controller),
so to get started let's create a simple controller:

```rust
use rwf::prelude::*;

#[controller]
async fn index() -> Response {
    Response::new().html("<h1>My first Rwf app!</h1>")
}
```

`rwf::prelude::*` includes most types, traits and functions you'll need to build applications.
Adding this declaration to your source code will make things easier, but it's not required.

`#[controller]` macro creates a controller from any async function that returns a [`Response`](controllers/response.md).

## Launch the server

With a controller ready to go, let's create a route and launch the Rwf HTTP server:

```rust
use rwf::http::{self, Server};

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    // Configure then logger.
    Logger::init();

    // Define routes.
    let routes = vec![
        route!("/" => index),
    ];

    // Launch the HTTP server.
    Server::new(routes)
        .launch("0.0.0.0:8000")
        .await
}
```

Your application is ready. You can launch it with Cargo:

```
cargo run
```

Once the server is running, your web application will be available on [http://localhost:8000](http://localhost:8000). The full code for this is available on [GitHub](https://github.com/levkk/rwf/tree/main/examples/quick-start).

## Learn more

- [Controllers](controllers/index.md)
- [Models](models/index.md)
- [Views](views/index.md)
- [Logging](logging.md)
