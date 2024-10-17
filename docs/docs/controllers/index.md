# Controller basics

Rwf comes with multiple pre-build controllers that can be used out of the box, for example to handle WebSocket connections, REST-style interactions, or serving static files. For everything else, the `Controller` trait can be implemented to handle any kind of HTTP requests.

## What's a controller?

The controller is the **C** in MVC: it handles user interactions with the web app and performs actions on their behalf. A controller takes care of user inputs, like forms, and all other HTTP requests to the app.

## Writing a controller

A controller is a plain Rust struct which implements the [`Controller`](https://docs.rs/rwf/latest/rwf/controller/trait.Controller.html) trait. As an example, let's write a controller which returns the current time in UTC.

#### Import types

```rust
use rwf::prelude::*;
```

The prelude module contains most of the types and traits necessary to work with Rwf. Including it will save you time and effort when writing code, but it's not required.

#### Define the struct

```rust
#[derive(Default)]
struct CurrentTime;
```

This struct has no fields, but you can add any internal state you want to keep track of in there. The `Default` trait is derived automatically to provide a convenient way to instantiate it.

#### Implement the `Controller` trait

```rust
#[async_trait]
impl Controller for CurrentTime {
    /// This function handles incoming HTTP requests.
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

The `Controller` trait is asynchronous. Support for async traits in Rust is still incomplete, so we use the [`async_trait`](https://docs.rs/async_trait) library to make it easy to use. The trait itself has a few methods, most of which have reasonable defaults. The only method that needs to be written by hand is `async fn handle()`.

#### `handle`

The `handle` method accepts a [`Request`](request) and must return a [`Response`](response). The response can be any valid HTTP response, including `404` or even `500`.

##### Errors

If an error occurs inside the `async fn handle` function, Rwf will return HTTP `500` automatically and display the error to the client.


## Connecting controllers

Once you implement a controller, adding it to the app requires mapping it to a route. A route is a unique URL, starting at the root of the app. For example, `/signup` is a route which could map to the `Signup` controller, and allow your users to create accounts.

Adding controllers to the app happens at server startup. A server can be launched from an async task anywhere in the code, but typically is done so from the `main` function:

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
    The `route!` macro is a shorthand for calling `CurrentTime::default().route("/time")`. We use it because it looks cool, but it's not required.
    You can instantiate your controller struct in any way you need, and call the `Controller::route` method when adding it to the server. Alternatively, you can implement the `Default` trait like we did in this example and use the macro.

### Test with cURL

Once the server is up and running, you can test your endpoints with cURL (or with a regular browser, like [Firefox](https://firefox.com)):

=== "cURL"
    ```bash
    curl localhost:8000/time -w '\n'
    ```
=== "Output"
    ```
    The current time is: 2024-10-17 0:23:34.6191103 +00:00:00
    ```


## Learn more

Read more about working with controllers, requests, and responses:

- [Requests](request)
- [Responses](response)
