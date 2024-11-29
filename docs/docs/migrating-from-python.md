# Migrating from Python

Rwf is written in Rust, so if you have an existing application you want to migrate to Rust, you have options. Rwf comes with its own [WSGI](https://en.wikipedia.org/wiki/Web_Server_Gateway_Interface) server, so you can run your existing Django or Flask apps without modifications, side by side with Rwf [controllers](controllers/index.md).

## Using WSGI

!!! note
    Rwf WSGI server is still experimental and is not as advanced as the popular [uWSGI](https://uwsgi-docs.readthedocs.io/en/latest/) project.

Adding a WSGI application to your Rwf server is pretty straight forward. First, make sure that the Python project is in your `PYTHONPATH`, for example:

```bash
export PYTHONPATH=/path/to/python/project
```

Rwf will load your Python app directly into its own memory (using [pyo3](https://docs.rs/pyo3)), so it needs to be able to find it when importing your app modules.

### Django

Django applications come with a WSGI interface, which Rwf can use directly. Usually, the interface is located in its own file, e.g. `project/wsgi.py`. The `WsgiController` accepts the Python module as an argument on initialization, in this case, `project.wsgi`.

Once initialized, the controller can be added into the server, and mounted on the `/*` (root, wildcard) path. This ensures that all requests are handled by Django:

```rust
use rwf::prelude::*;
use rwf::http::Server;
use rwf::controller::WsgiController;

#[tokio::main]
async fn main() {
    Server::new(vec![
        WsgiController::new("project.wsgi")
            .wildcard("/"),
    ])
    .launch()
    .await
    .unwrap();
}
```

### Python dependencies

Your application most likely has other dependencies, e.g. `django`, or `Flask` packages, and many more. To make sure they work when loaded into Rwf, either create and activate a virtual environment before launching the server, or install those packages globally (e.g., when deploying with Docker).

## Moving traffic to Rust

As you rewrite your endpoints to use Rwf and Rust, you can move traffic one route at time without disrupting your users. For example, if you are ready to move the route `/users` to Rust, add the controller for it into the server:

```rust
/// Your new "/users" controller
/// written in Rust.
use crate::controllers::Users;

#[tokio::main]
async fn main() {
    Server::new(vec![
        WsgiController::new("project.wsgi")
            .wildcard("/"),
        route!("/users" => Users),
    ])
    .launch()
    .await
    .unwrap()
}
```

Rwf routing algorithm will match requests to `/users` to the `Users` controller instead of sending it to WSGI, because the `Users` controller path is more specific and has higher priority than wildcard routes.

## Learn more

- [examples/django](https://github.com/levkk/rwf/tree/main/examples/django)
