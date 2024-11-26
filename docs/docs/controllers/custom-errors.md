# Custom error pages

When a controller returns an HTTP error code higher than 399, like HTTP 404 or HTTP 500 for example, Rwf displays an error page.
The default error page doesn't provide much information beyond the error code and name, so it's a good idea to use custom ones that would direct your users to helpful resources.

## Custom 404

The most common error your users will run into is requesting a page that doesn't exist. Customizing this error message can be done by providing a catch-all (or wildcard) controller:

```rust
use rwf::prelude::*;

#[derive(Default)]
struct NotFound;

#[async_trait]
impl Controller for NotFound {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        render!(request, "templates/404.html")
    }
}
```

To configure the Rwf HTTP server to serve your custom `404.html` template, map your `NotFound` controller to the `/*` route, like so:

```rust
Server::new(vec![
    // ... your other routes ...
    NotFound::default().wildcard("/"),
]);
```

Wildcard routes have a low rank of -20. When matching requests to controllers, the HTTP server will attempt all other routes, and if none of them match, it will serve the wildcard route.

!!! note
    If your controllers return HTTP 404 manually, the server will not use your wildcard route and will
    return the default error page instead. Universal catchers for error codes are on the roadmap.
