# Getting started

Rust Web Framework (Rwf for short) is set of libraries and tools to build web applications using the Rust programming langauge. It aims to be comprehensive, by providing all features
for you to build modern, fast, and secure web apps, out of the box.

Rwf has very few dependencies, and is easy to install and use within new or existing Rust applications.

## Install Rust

If you haven't already, install the Rust compiler and tools from [rust-lang.org](https://rust-lang.org). Rwf doesn't use any nightly or experimental features,
so the stable version of the compiler will work.

## Create a project

Rwf can be used inside any Rust binary or library project. If you don't have a project already, you can create one with Cargo:

```bash
cargo init --bin rwf-web-app
```

## Install Rwf

Rwf has two packages:

* `rwf` which is the Rust crate[^1] used to build web apps
* `rwf-cli` which is a binary application that helps manage Rust projects built with Rwf

To install them, run the following while inside the root directory of your Cargo-created project:

```
cargo add rwf
cargo install rwf-cli
```

[^1]: A "crate" is a Rust package used as a dependency in other packages. It's analogous to "package" in JavaScript or Python.

## Building the app

With the packages installed, you're ready to launch your first web app in Rust. Rwf is built using the [MVC](https://en.wikipedia.org/wiki/Model%E2%80%93view%E2%80%93controller) (Model-view-controller) design pattern,
so to get started, let's create a simple controller that will serve the index page (`/`) of your app:

```rust
use rwf::prelude::*;

#[derive(Default)]
struct Index;

#[async_trait]
impl Controller for Index {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::new().html("<h1>My first Rwf app!</h1>"))
    }
}
```

`rwf::prelude::*` includes the vast majority of types, structs, traits and functions you'll be using when building controllers with Rwf.
Adding this declaration in your source files will make handling imports easier, but it's not required.

Rwf controllers are defined as Rust structs which implement the [`Controller`](../controllers/) trait. The trait is asynchronous, hence the `#[async_trait]` macro[^2],
and has only one method you need to implement: `async fn handle`. This method
accepts a [`Request`](../controllers/request), and must return a [`Response`](../controllers/response).

In this example, we are returning HTTP `200 - OK` with the body `<h1>My first Rwf app</h1>`. This is not strictly valid HTML,
but it'll work in all browsers for our demo purposes.

[^2]: The Rust language support for async traits is still incomplete. The `async_trait` crate helps with writing async traits in an ergonomic way.

## Launching the server

Once you have at least one controller, you can add it to the Rwf HTTP server and launch it on the address and port of your choosing:

```rust
use rwf::http::{self, Server};

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    // Configure then logger (stderr with colors by default)
    Logger::init();

    Server::new(vec![
        route!("/" => Index),
    ])
    .launch("0.0.0.0:8000")
    .await
}
```

Rwf uses the `log` crate for logging. `Logger::init()` automatically configures it for your app using `tracing-subscriber`, but if you prefer, you can configure logging yourself
using the crate of your choosing.

Launching the server can be done with Cargo:

```
cargo run
```

Once the server is running, you can visit the index page by pointing your browser to [http://localhost:8000](http://localhost:8000).

Full code for this is available in GitHub in [examples/quick-start](https://github.com/levkk/rwf/tree/main/examples/quick-start).

## Learn more

- [Controllers](controllers/)
- [Models](models/)
- [Views](views/)
