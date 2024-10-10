
# RESTful framework

Rum comes with a REST framework (just like Django REST Framework) built-in. Serialization is automatically done with JSON (using `serde_json`) and the API follows the standard CRUD (create, read, update, destroy) pattern.

## Adding REST controllers

To add a REST controller to your Rum app, your controller needs to implement the `ModelController` trait:

```rust
#[derive(rum::macros::ModelController)]
struct UsersController;

#[async_trait]
impl ModelController for UsersController {
    type Model = User;
}
```

The model needs to be serializable into and from JSON, so make sure to derive the appropriate serde traits:

```rust
use serde::{Serialize, Deserialize};

#[derive(Clone, rust::macros::Model, Serialize, Deserialize)]
struct User {
    // Hide this field entirely from the API.
    #[serde(skip_deserializing)]
    id: Option<i64>,

    // The only required field at the API.
    email: String,

    #[serde(with = "time::serde::iso8601", default = "OffsetDateTime::now_utc")]
    created_at: OffsetDateTime,

    #[serde(default="bool::default")]
    admin: bool,
}
```

Adding the controller to the server is then simple:

```rust
#[tokio::main]
async fn main() {
    Server::new(vec![
        UsersController::default().crud("/api/users"),
    ])
    .launch("0.0.0.0:8000")
    .expect("failed to shut down server");
}
```

The `crud` method will automatically implement the following:

| Path | Method | Description |
|------|--------|-------------|
| `/api/users` | GET | List all users. Supports pagination, e.g. `?page_size=25&page=1`. Default page size is 25.|
| `/api/users/:id` | GET | Fetch a user by primary key. |
| `/api/users`| POST | Create a new user. All fields not marked optional or not having serde-specified defaults are required. |
| `/api/users/:id` | PUT | Update a user. Same requirement for fields as the create method above. |
| `/api/users/:id` | PATCH | Update a user. Only the fields that have changed can be supplied. |


You can test this with cURL:

```
$ curl localhost:8000/api/users -d '{"email": "test@test.com"}' -w '\n'

{"email":"test@test.com","created_at":"+002024-10-09T22:59:10.693321000Z","admin":false}
```

## Customizing serialization

Serde allows full control over how fields are serialized and deserialized, including rewriting, renaming, and skipping fields entirely. See [Serde documentation](https://serde.rs/field-attrs.html) for more details.

## Writing your own REST controller

You can write your own REST controller by implementing the `RestController` trait. See [the code](/rum/src/controller/mod.rs) for details.