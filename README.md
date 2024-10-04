# Rum &dash; Complete Web Framework

Rum is a comprehensive framework for building web applications in Rust. Built using the classic MVC  pattern (model-view-controller), Rum comes standard with everything you need to easily build fast and secure web apps.

### Features overview

- :heavy_check_mark: HTTP server
- :heavy_check_mark: User-friendly ORM to build PostgreSQL queries easily
- :heavy_check_mark: Dynamic templates
- :heavy_check_mark: Authentication & built-in user sessions
- :heavy_check_mark: Middleware
- :heavy_check_mark: Background jobs
- :heavy_check_mark: WebSockets support
- :heavy_check_mark: Environment-specific configuration
- :heavy_check_mark: Logging and metrics

### Quick start

To add Rum to your stack, create a Rust binary application and add `rum_framework` to your dependencies:

```
cargo add rum-framework
```

Building an app is then as simple as:

```rust
use rum::prelude::*;

#[derive(Default)]
struct IndexController;

#[rum::async_trait]
impl Controller for IndexController {
	async fn handle(&self, request: &Request) -> Result<Response, Error> {
		Ok(Response::new().html("<h1>Hey Rum!</h1>"))
	}
}

#[rum::tokio::main]
async fn main() {
	Server::new(vec![
		IndexController::default().route("/"),
	])
	.launch("0.0.0.0:8000")
	.await
	.expect("error shutting down server");
}
```