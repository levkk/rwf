# HTTP server

Rwf comes with its own HTTP server. The server handles all incoming requests and routes them to a controller. The server is built with Tokio and can handle millions of connections.

## Basic controller

All Rwf controllers have the same structure: they accept a `rwf::http::Request` and must return a `rwf::http::Response`. The rest of the implementation is up to you. For example, if you want to display a page at the root of your site, you'll need to do two things:

- create a controller which will return the HTML you want to display
- route the `/` path to that controller

Creating the controller requires defining a struct and implementing the `rwf::controller::Controller` trait for that struct:

```rust
use rwf::prelude::*;

#[derive(Default)]
struct IndexController;

#[async_trait]
impl Controller for IndexController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::new().html("
            <!DOCTYPE html>
            <html>
              <body>
                <h1>Hello</h1>
              </body
            ></html>
        "))
    }
}
```

Once you have a controller, routing a request to that controller can be done at server start:

```rust
use rwf::http::Server;
use rwf::macros::route;

#[tokio::main]
async fn main() {
    Server::new(vec![
        route!("/" => IndexController),
    ])
    .launch()
    .await
    .unwrap();
}
```

## Building web apps

Returning HTML is great, but most web apps also accept user input and do something with it. To help with that, Rwf comes with a set of additional controllers and tools that manage that complexity.

### Handling forms

A typical form sends data via POST. While you could use the `Controller::handle` to check for the request type and act accordingly, it's easier to let Rwf do a bit of work for you by implementing the `PageController` instead:

```rust
use rwf::controllers::PageController;

#[derive(rwf::macros::PageController)]
struct SignupController;

#[async_trait]
impl PageController for SignupController {
    async fn get(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::new().html(r#"
            <!DOCTYPE html>
            <html>
              <body>
                <form action="/signup" method="post">
                  <%= csrf_token() %>
                  <label>Email</labelL
                  <input name="email" type="email" required>
                  <button type="submit">Sign up</button>
                </form>
              </body>
            </html>
        "#))
    }

    async fn post(request: &Request) -> Result<Response, Error> {
        let form_data = request.form_data();
        let email = form_data.get::<String>("email");

        // Create the user
        Ok(Response::new().redirect("/welcome"))
    }
}
```

The `PageController` implements the `handle` method of the `Controller` and splits the incoming requests based on the HTTP request method: if it's a GET request, the `get` method is called, and if it's a POST request, the `post` method is called. If any other method is used, e.g. PATCH or PUT, a `405 Method Not Allowed` code is returned.

#### Type-safe forms

Extracting multiple fields from a form could be tedious, so Rwf comes with a few helpers:

```rust
#[derive(rwf::macros::Form)]
struct SignupForm {
    email: String,
    password: String,
    password2: String,
}

let form = request.form::<SignupForm>()?;
```

This will automatically extract the `email`, `password` and `password2` fields from the request FormData and map them to the struct. If any of the fields are missing or are of the wrong type, a `400 - Bad Request` will be returned automatically.

If you're in the HTML-over-the-wire camp, `PageController` and `Controller` will handle the vast majority of your use cases. However, if you prefer to build your frontends in JavaScript, Rwf comes with a couple more controllers that will come in handy.

## More examples

See [Rwf + Turbo](/examples/turbo) for a complete example of building a Single Page Application with Rwf, Turbo, Stimulus and WebSockets.
