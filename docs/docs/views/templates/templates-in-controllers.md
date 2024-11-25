# Templates in controllers

Using templates in [controllers](../../controllers/index.md) typically involves rendering them inside a request handler and returning the result as HTML, for example:

```rust
struct Index;

#[async_trait]
impl Controller for Index {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let template = Template::load("templates/index.html")?;
        let ctx = context!("title" => "Home page");
        let rendered = template.render(&ctx)?;

        Ok(Response::new().html(rendered))
    }
}
```

The template will be loaded from the [template cache](caching.md), rendered with the provided context, and used as a body for a response with the correct `Content-Type` header.

## Render macro
Since it's very common to render templates inside controllers, Rwf has the `render!` macro to make it less verbose:

```rust
#[async_trait]
impl Controller for Index {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        render!(request, "templates/index.html", "title" => "Home page")
    }
}
```

The `render!` macro takes the request as the first argument, the template path, and optionally a mapping of variable names and values. It returns a [`Response`](../../controllers/response.md) automatically.

If the template doesn't have any variables, you can omit them:

```rust
render!(request, "templates/index.html")
```

Passing the request into the macro ensures that secure [CSRF](../../security/CSRF.md) protection tokens are generated automatically.

### Response code

By default, the `render!` macro returns the rendered template with HTTP code `200 OK`. If you want to return a different code, pass it as the last argument to the macro:

```rust
render!(request, "templates/index.html", "title" => "Home page", 201)
```
