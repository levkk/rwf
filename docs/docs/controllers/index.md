# Controller basics

Rwf comes with multiple pre-build controllers that can be used out of the box, for example to handle WebSocket connections, REST-style interactions, or serving static files. For everything else, the `Controller` trait can be implemented to handle any kind of HTTP requests.

## What's a controller?

The controller is the **C** in MVC: it handles user interactions with the web app and performs actions on their behalf. User inputs, like forms, and requests to the web app via HTTP, is taken care of by controllers.

## Writing a controller

A controller is a plain Rust struct which implements the `rwf::controller::Controller` trait. For example, a simple controller which responds with the current time can be with a few simple steps.

#### Import types

```rust
use rwf::prelude::*;
```

The prelude module contains most of the types and traits necessary to work with Rwf. Including it will save you time and effort when writing code.

#### Define the struct

```rust
#[derive(Default)]
struct CurrentTime;
```

A controller is any Rust struct that implements the `Controller` trait. The `Default` trait is derived automatically to provide a convenient way to instantiate it.

#### Implement the `Controller` trait

```rust
#[async_trait]
impl Controller for CurrentTime {
    /// This function responds to all incoming HTTP requests.
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let time = OffsetDateTime::now_utc();

        // This creates an HTTP "200 OK" response,
        // with "Content-Type: text/plain" header.
        let response = Response::new()
            .text(format!("The current time is: {:?}", time));

        Ok(response)
    }
}
```

The `Controller` trait is asynchronous. Support for async traits in Rust is still incomplete, so we use the `async_trait` package to make it easy to use. The trait itself has a few methods, most of which have reasonable defaults. The only method
that needs to be written by hand is `async fn handle()`.

#### `handle`

The `handle` method accepts [`rwf::http::Request`](https://docs.rs/rwf/latest/rwf/http/request/struct.Request.html) and must return [`rwf::http::Response`](https://docs.rs/rwf/latest/rwf/http/response/struct.Response.html). The response can be any valid HTTP response, including `404` or even `500`.
See [Request](request) documentation for examples of requests, and [Response](response) documentation for more information on creating responses.


## Connecting controllers

Once you have a controller, adding it to the app requires mapping it to a route. A route is a unique URL, starting at the root of the app. For example, a route displaying all the users in our app could be `/app`, which would be handled by the `Users` controller.

Adding controllers to the app happens at server startup. A serve can be launched from anywhere in the code, but typically is done so in the `main` function.

```rust
use rwf::prelude::*;
use rwf::http::{self, Server};

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    Server::new(vec![
        // Map the `/time` route to the `CurrentTime` controller.
        route!("/time" => CurrentTime),
    ])
    .launch("0.0.0.0:8000")
    .await
}
```

!!! note
    The `route!` macro is a shorthand for calling `CurrentTime::default().route("/time")` and it looks pretty.
    You can instantiate your controller struct in any way you need, and call the `Controller::route` method when you're ready to add it to the server.

    You can also implement the `Default` trait for your controller and continue to use the macro.

### Test with cURL

Once the server is running, you can test your endpoints with cURL (or with a regular browser, like [Firefox](https://firefox.com)):

=== "cURL"
    ```bash
    curl localhost:8000/time -w '\n'
    ```
=== "Output"
    ```
    The current time is: 2024-10-17 0:23:34.6191103 +00:00:00
    ```
