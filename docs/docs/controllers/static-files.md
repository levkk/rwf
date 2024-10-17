# Static files

Rwf comes with a static files server built-in. It will handle serving files out of any directory
and will automatically return the right `Content-Type` header (also known as [MIME](https://developer.mozilla.org/en-US/docs/Web/HTTP/MIME_types)), based on the file extension.

## Serve static files

The static files server is just another [controller](../), implemented internally. To add it to your app, you can
add it to the server at startup:

```rust
use rwf::controller::StaticFiles;
use rwf::http::{Server, self};

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    let server = Server::new(vec![
        StaticFiles::serve("static")?,
    ])
    .launch("0.0.0.0:8000")
    .await
}
```

This example will serve all static files in the `static` directory under the `/static` route.
