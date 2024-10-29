
# Middleware

Rwf's middleware is inspired by Django's middleware system and allows to rewrite requests, responses, and to intercept requests to controllers entirely.

## Writing middleware

Writing your middleware is simple and requires only implementing the `Middleware` trait. The trait provides a way to parse requests, change their contents, and either forward them down the middleware chain or to stop the request and return a response.


```rust
use rwf::controller::middleware::prelude::*;

#[derive(Default)]
struct OnlyLinuxBrowsers;

#[rwf::async_trait]
impl Middleware for OnlyLinuxBrowsers {
    async fn handle_request(&self, request: Request) -> Result<Outcome, Error> {
        if let Some(header) = request.headers().get("user-agent") {
            if header.contains("Linux") {
                request
                    .headers_mut()
                    .insert("X-This-Year", "Is of the Linux Desktop")l
                return Ok(Outcome::Forward(request));
            }
        }

        return Ok(Outcome::Stop((request, Response::redirect("https://archlinux.org"))))
    }
}
```

## Adding middleware to controllers

Adding middleware to controllers can be done by implementing the `middleware` method on the controller:

```rust
struct WindowsController {
    middleware: MiddlewareSet,
}

impl WindowsController {
    fn new() -> Self {
        Self {
            middleware: MiddlewareSet::new(vec![
                OnlyLinuxBrowsers::default().middleware(),
            ])
        }
    }
}

#[rwf::async_trait]
impl Controller for WindowsController {
    fn middleware(&self) -> &MiddlewareSet {
        &self.middlware
    }
}
```

## Order of evaluation

Middleware is evaluated in the order it's added to the middleware set. The middleware modifying requests is evaluated first to last, while middleware modifying responses is evaluated last to first.

## Modifying responses

To modify responses, implement the `handle_response` method on the `Middleware` trait. See the included [request rate limiter](rwf/src/controller/middleware/rate_limiter.rs) middleware for complete example.
