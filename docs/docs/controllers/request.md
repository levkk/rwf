# Requests

For each HTTP request served by Rwf, a new [`Request`](https://docs.rs/rwf/latest/rwf/http/request/struct.Request.html) struct is created. It contains the client IP address,
browser headers, [cookies](../cookies), [session](../sessions) information, and the request body.

## Headers

Fetching headers sent by the client in the HTTP request can be done by calling the `headers` method on the request object
inside a controller:

```rust
struct Index;

impl Controller for Index {
    // Handle HTTP request.
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        // Get the `Accept` header from the request.
        let accept = request
            .headers()
            .get("accept");

        if let Some(accept) = accept {
            Ok(Response::new().text(format!("Accept: {}", accept)));
        } else {
            Ok(Response::bad_request())
        }
    }
}
```

!!! note
    Headers in Rwf are case-insensitive, so `accept` and `Accept` are equivalent.

Most browsers send required headers like `Origin`, `Accept`, and `User-Agent`, but that doesn't mean all HTTP clients will.
Checking for valid headers is good practice to avoid bad actors like bots. Read more about intercepting HTTP requests with [Middleware](../middleware).

## Request body

For requests that include a body, like `POST` or `PUT`, the body can be read using multiple methods, depending
on the expected content type.

### Forms

HTTP forms submitted using `POST` (or `PUT`/`PATCH`) are encoded using either URL encoding or multipart encoding.
Parsing the form data is automatically handled by Rwf, so accessing a form field can be done in a couple ways.

#### Form fields

```rust
let form = request.form_data();
let email = form.get::<String>("email");

if let Some(email) = email {
    // Create account.
}
```

Form fields are converted to a Rust type manually, by passing in the data type to
the generic [`FormData::get`](https://docs.rs/rwf/latest/rwf/http/form_data/enum.FormData.html#method.get) function.
All data types that implement the [`FromStr`](https://doc.rust-lang.org/stable/std/str/trait.FromStr.html) trait are supported, including integers, floats, boolean, and UUIDs.

#### Strictly-typed forms

Instead of parsing form fields manually on each request, you can define a Rust struct with the matching
column names and data types to your form:

=== "Rust"
    ```rust
    #[derive(Debug, macros::Form)]
    struct UserForm {
        // required
        email: String,
        // required
        password: String,
        // optional
        password2: Option<String>,
    }

    let form = request.form::<UserForm>()?;

    if form.password2.is_none() {
      return Ok(Response::bad_request());
    }
    ```
=== "HTML"
    ```html
    <form>
      <input name="email" type="text" required>
      <input name="password" type="password" required>
      <input name="password2" type="password">
    </form>
    ```

### JSON

If the body is expected to be JSON, it can be read using the `json` method instead. The `json` method
is generic and automatically converts the request body into a Rust struct using the `serde_json` crate:

=== "Rust"
    ```rust
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct User {
        email: String,
    }

    let user = request.json::<User>()?;
    ```
=== "JSON"
    ```json
    {
      "email": "new-user@example.com"
    }
    ```

#### Unstructured JSON

If you don't know the schema of the JSON request, you can use [`json_raw`](https://docs.rs/rwf/latest/rwf/http/request/struct.Request.html#method.json_raw) instead, for example:

=== "Rust"
    ```rust
    let json = request.json_raw()?;
    println!("{}", json["id"]);
    ```
=== "JSON"
    ```json
    {
      "id": 5,
      "name": "New user"
    }
    ```

### Parsing errors

If you use [`FormData::get_required`](https://docs.rs/rwf/latest/rwf/http/form_data/enum.FormData.html#method.get_required) or [`Request::json`](https://docs.rs/rwf/latest/rwf/http/request/struct.Request.html#method.json) methods with the `?` operator,
an error will be returned to the client automatically if the parsing of the form data fails.
Unlike other controller errors that return HTTP `500`, this type of error will return HTTP `400` (Bad Request).
