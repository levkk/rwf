# Responses

Each HTTP request served by Rwf is expected to return a response. If your app is using a [REST](REST/index.md) API, responses
are typically JSON. If you prefer [HTML over the wire](../views/turbo/index.md) or plain old websites, the responses will contain HTML or text.

## Creating responses

To create a response, you can just instantiate the [`Response`](https://docs.rs/rwf/latest/rwf/http/response/struct.Response.html) struct and populate the body
with the right content. The most popular response types have their own instantiation methods in Rwf:

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
  .header("Content-Type", "application/octet-stream")
  .body(mystery_bytes);
```

The `Content-Length` header is always set automatically, but if you absolutely need to, you can set it manually using the [`header`](https://docs.rs/rwf/latest/rwf/http/response/struct.Response.html#method.header) method.

### Headers

Setting custom headers can be done with the [`header`](https://docs.rs/rwf/latest/rwf/http/response/struct.Response.html#method.header) method:

```rust
let response = Response::new()
  .header("X-My-Header", "My value")
  .header("Cache", "no-store");
```

## Learn more

- [Cookies](cookies.md)
- [Sessions](sessions.md)
