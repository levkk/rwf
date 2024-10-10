
# RESTful framework

Rum comes with a REST framework (just like Django REST Framework) built-in. Serialization is automatically done with JSON (using `serde_json`) and the API follows the standard CRUD (create, read, update, destroy) operations.

## Adding REST controllers

There are two ways to add a REST controller to your app: implementing the `RestController` trait manually for each supported method, or implementing the `ModelController` trait.

### `ModelController`

The `ModelController` trait works with Rum's ORM models and automatically (de)serializes API inputs/outputs and fetches and updates database records.

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

The `crud` method will automatically implement the following routes:

| Path | Method | Description |
|------|--------|-------------|
| `/api/users` | GET | List all users. Supports pagination, e.g. `?page_size=25&page=1`. Default page size is 25.|
| `/api/users/:id` | GET | Fetch a user by primary key. |
| `/api/users`| POST | Create a new user. All fields not marked optional or not having serde-specified defaults are required. |
| `/api/users/:id` | PUT | Update a user. Same requirement for fields as the create method above. |
| `/api/users/:id` | PATCH | Update a user. Only the fields that have changed can be supplied. |


The DELETE method is not implemented yet, see [ROADMAP](/ROADMAP.md).

### Testing

You can test this with cURL (or your favorite API test client, e.g. Postman):

```
$ curl localhost:8000/api/users -d '{"email": "test@test.com"}' -w '\n'
{"id":1, email":"test@test.com","created_at":"+002024-10-09T22:59:10.693321000Z","admin":false}
```

### `RestController`

The `RestController` parses incoming requests and splits them based on the path and the request method to one of the 5 RESTful methods:

- `list` resources
- `get` a resource
- `update` a resource
- `patch` a resource
- `delete` a resource

The methods for each default to return a `501 - Not Implemented` response, so if you want to support some or all of them, you'll need to implement those trait methods:

```rust
#[derive(rum::macros::RestController, Default)]
struct MyController;

#[rum::async_trait]
impl RestController for MyController {
    type Resource = i64; // Use integers as the resource identifiers.
                         // Can be any other data type that implements `rum::controller::ToParameter` trait.

    /// GET /
    async fn list(&self, _request: &Request) -> Result<Response, Error> {
        let result = serde_json::json!([
            {"id": 5, "email": "test@test.com"},
            {"id": 7, "email": "hello@test.com"},
        ]);

        Ok(Response::new().json(result)?)
    }

    /// GET /:id
    async fn get(&self, _request: &Request, id: &Self::Resource) -> Result<Response, Error> {
        let result = serde_json::json!({
            "id": *id,
            "email": "guest@test.com",
        });

        Ok(Response::new().json(result)?)
    }

    // All other methods will return HTTP 501.
}
```

Adding this controller to the server is then possible with:

```rust
Server::new(vec![
    MyController::default().rest("/api/rest")
])
```

The `rest` method will create the paths to serve all 5 REST verbs, just like the `ModelController` except the verbs are implemented manually.

## Customizing serialization

Serde allows full control over how fields are serialized and deserialized, including rewriting, renaming, and skipping fields entirely. See [Serde documentation](https://serde.rs/field-attrs.html) for more details.
