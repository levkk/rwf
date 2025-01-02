# Responses

Each HTTP request served by Rwf is expected to return a response. If your app is using a [REST](REST/index.md) API, responses
are typically JSON. If you prefer [HTML over the wire](../views/turbo/index.md) or plain old websites, the responses will contain HTML or text.

## Creating responses

To create a response, you can just instantiate the [`Response`](https://docs.rs/rwf/latest/rwf/http/response/struct.Response.html) struct and populate the body
with the right content. The most popular response types have their own instantiation methods:

=== "HTML"
    ```rust
    let response = Response::new()
      .html("<h1>Big letters!</h1>");
    ```
=== "JSON"
    ```rust
    let json = serde_json::json!({
      "id": 5,
      "email": "test@example.com"
    });

    let response = Response::new().json(json)?;
    ```
=== "Plain text"
    ```rust
    let response = Response::new()
      .text("One apple a day keeps the doctor away!");
    ```

Using one of those methods will automatically set the right `Content-Type` and `Content-Length` headers.

### Raw data

If your endpoint is sending binary data or some data type we don't have a method for, you can always set the body and content type manually:

```rust
let mystery_bytes: Vec<u8> = vec![1, 1, 2, 3, 5, 8, 13];

let response = Response::new()
  .body(mystery_bytes)
  .header("Content-Type", "x-application/fibonacci");
```

!!! note
    `Response` attempts to deduce the `Content-Type` by the body type, so if you want to override its decision,
    set the header _after_ setting the body on the response. By default, `Vec<u8>` uses the `Content-Type: application/octet-stream`.

The `Content-Length` header is always set automatically, but if you absolutely need to, you can set it [manually](#headers).

### Headers

Setting custom headers can be done with the [`header`](https://docs.rs/rwf/latest/rwf/http/response/struct.Response.html#method.header) method, for example:

```rust
let response = Response::new()
  .header("X-My-Header", "My value")
  .header("Cache", "no-store");
```

Headers are rewritten to lowercase lettering, i.e. `X-My-Header` and `x-my-header` are equivalent.

### HTTP codes

A `Response` returns with HTTP code `200 - OK` by default. If you want to set a different code, you can:

```rust
let response = Response::new()
    .html("<h1>Created!</h1>")
    .code(201);
```

Common use cases have their own methods to make this easier.

#### Redirect

Redirecting the user to a different URL can be done with:

```rust
let response = Response::new()
    .redirect("/different-url");
```

This automatically sets the `Location` and `Cache-Control` headers, and returns with HTTP code `302 - Found`.

#### Errors

Common errors have their own methods which will return the correct HTTP response code and built-in response body.

##### 401 - Unauthorized

When your users have failed some authentication challenge, you can block access to a resource with HTTP response code `401 - Unauthorized`:

```rust
let resonse = Response::unauthorized();
```

Use this one if your frontend can handle it gracefully. If not, a gentle [redirect](#redirect) to your login page may be preferable.

##### 403 - Forbidden

When your user is logged in, but doesn't have access to the request resource, you can block access to it with HTTP response code `403 - Forbidden`:

```rust
let resonse = Response::forbidden();
```

##### 404 - Not found

Commonly used when some resource doesn't exist, HTTP response code `404 - Not Found` can be returned with:

```rust
let response = Response::not_found();
```

HTTP 404 is returned automatically by Rwf when a user requests a route that doesn't have a controller.

## Syntactic sugar

Returning certain types of responses is common, so Rwf has a few automatic conversions to remove boilerplate from controllers. In the context of a controller method, the following statements are equivalent.

##### HTML

=== "Shortcut"
    ```rust
    "<h1>Text</h1>".into()
    ```
=== "Code"
    ```rust
    Response::new().html("<h1>Text</h1>")
    ```

##### JSON

=== "Shortcut"
    ```rust
    serde_json::json!({"hello": "world"}).into()
    ```
=== "Code"
    ```rust
    Response::new().json(serde_json::json!({"hello": "world"})?;
    ```

## Learn more

- [Cookies](cookies.md)
- [Sessions](sessions.md)
