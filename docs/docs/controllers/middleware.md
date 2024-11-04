# Middleware

Middleware runs before a request reaches a controller and has the ability to modify the request, or block it from reaching the controller entirely. Middleware is used to validate
incoming requests against some conditions, e.g. required headers. It can also be used to mark requests with special tags, by adding custom headers.

## Using middleware

Middleware needs to be specified on each controller. By default, all controllers come with no middleware, so requests processed by them are unmodified from their original state.

### Define middleware

Middleware, similar to [controllers](index.md), is any struct which implements the [`Middleware`](https://docs.rs/rwf/latest/rwf/controller/middleware/trait.Middleware.html) trait. The only method that needs
implementation is the [`async fn handle_request`](https://docs.rs/rwf/latest/rwf/controller/middleware/trait.Middleware.html#tymethod.handle_request) method, which accepts a [`Request`](request.md) and must return an [`Outcome`](https://docs.rs/rwf/latest/rwf/controller/middleware/enum.Outcome.html).

If the request is allowed to proceed, [`Outcome::Forward`](https://docs.rs/rwf/latest/rwf/controller/middleware/enum.Outcome.html#variant.Forward) is returned, containing the request, in its modified or unchanged form.
If on the other hand, the request failed some kind of validation, [`Outcome::Stop`](https://docs.rs/rwf/latest/rwf/controller/middleware/enum.Outcome.html#variant.Stop) must be returned with a [`Response`](response.md), for example:

```rust
use rwf::controller::middleware::prelude::*;

struct RequiredHeaders {
    headers: Vec<String>,
}

impl Default for RequiredHeaders {
    fn default() -> Self {
        Self {
            headers: vec![
                "X-Request-Id".to_string()
            ],
        }
    }
}

#[async_trait]
impl Middleware for RequiredHeaders {
    async fn handle_request(&self, request: Request) -> Result<Outcome, Error> {
        for header in &self.headers {
            let header = request.headers().get(header);

            if header.is_none() {
                return Ok(Outcome::Stop(request, Response::bad_request()));
            }
        }

        Ok(Outcome::Forward(request))
    }
}
```

### Enable middleware

Enabling middleware needs to be done at the controller level. For each controller where you want the middleware
to run, add it to the struct fields and instantiate it when the controller is created:

```rust
struct Index {
    middleware: MiddlewareSet,
}

impl Default for Index {
    fn default() -> Self {
        Index {
            middleware: MiddlewareSet::new(vec![
                RequiredHeaders::default()
                    .middleware(),
            ])
        }
    }
}
```

When implementing the [`Controller`](https://docs.rs/rwf/latest/rwf/controller/trait.Controller.html) trait for your controller, implement the [`middleware`](https://docs.rs/rwf/latest/rwf/controller/trait.Controller.html#method.middleware) method as well:

```rust
#[async_trait]
impl Controller for Index {
    // This controller has middleware.
    fn middleware(&self) -> &MiddlewareSet {
        &self.middleware
    }

    // Middleware will run before this method.
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        /* ... */
    }
}
```

Adding a controller with middleware to the server requires no special code, since middleware is handled by the [`Controller`](https://docs.rs/rwf/latest/rwf/controller/trait.Controller.html) trait internally.
