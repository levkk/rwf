# Requests

For each HTTP request served by Rwf, a new [`Response`](https://docs.rs/rwf/latest/rwf/http/request/struct.Request.html) struct is created. It contains the client IP address,
browser headers, cookies, session information, and the request body.

## HTTP headers

Fetching headers sent by the client in the HTTP request can be done by calling the `headers` method on the request object
inside a controller:

```rust
let accept = request
    .headers()
    .get("accept")
    .unwrap();

assert_eq!(accept, "*/*");
```

!!! note
    Headers in Rwf are case-insensitive, so `accept` and `Accept` are equivalent.


## Request body

For requests that include a body, like `POST` or `PUT`, the body can be read using different methods, depending
on the expected content type.

### Forms

HTTP forms submitted using `POST` (or `PUT`/`PATCH`) are encoded using either URL encoding or chunked encoding.
Parsing the form data is automatically handled by Rwf, so accessing a form field can be done in one of two ways.

#### Form fields

```rust
let form = request.form_data();
let email = form.get::<String>("email");
```

Form fields are converted to a Rust type manually, by passing in the data type as a parameter to
the generic `get` function. All data types that implement the `std::str::FromStr` trait are supported,
including integers, floats, and UUIDs.

#### Strictly-typed forms

Instead of parsing form fields manually on each request, you can define a Rust struct with the matching
column names and data types to your form.

=== "Rust"
    ```rust
    #[derive(macros::Form)]
    struct UserForm {
        email: String,
        password: String,
        password2: Option<String>,
    }

    let form = request.form::<UserForm>()?;
    assert_eq!(form.email, "new-user@example.com");
    ```
=== "HTML"
    ```html
    <form>
      <input name="email" type="text" required>
      <input name="password" type="password" required>
      <input name="password2" type="password">
    </form>
    ```

For example, if the body is expected to be a form, it can be read using `form_data` method:

=== "Rust"
    ```rust
    let form_data = request.form_data();
    let email = form_data.get_required::<String>("email")?;

    assert_eq!(email, "new-user@example.com");
    ```
=== "HTML"
    ```html
    <form method="post">
        <input type="email" name="email" required>
    </form>
    ```

### Reading JSON

If the body is expected to be JSON, it can be read using the `json` method instead. The `json` method
is generic and automatically converts the request body in a Rust struct using the `serde_json` crate:

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct User {
    email: String,
}

let user = request.json::<User>()?;

assert_eq!(user.email, "new-user@example.com");"
```
