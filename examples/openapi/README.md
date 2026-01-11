
# REST framework

Rwf comes with a REST framework (just like Django REST Framework) built-in. Serialization is automatically done with JSON (using `serde_json`) and the API follows the standard CRUD (create, read, update, destroy) operations.

## OpenApi

Rwf offers an Option to auto generate OpenApi Specs for the CRUD handler (controller implementing `ModelController`) 

### `ModelController` with OpenApi Specs

The `ModelController` trait works with Rwf's ORM models and automatically (de)serializes API inputs/outputs and fetches and updates database records.

```rust
#[generate_openapi_model_controller(i64, User)]
#[derive(rwf::macros::ModelController)]
struct UsersController;

#[async_trait]
impl ModelController for UsersController {
    type Model = User;
}
```

The model needs to be serializable into and from JSON, so make sure to derive the appropriate serde traits.
The Model needs to implement the `ToSchema` and `ToResponse` trait to let Rwf generate the appropriate Specs:

```rust
use serde::{Serialize, Deserialize};
use rwf::prelude::{ToSchema, ToResponse};

#[derive(Clone, rust::macros::Model, Serialize, Deserialize, ToSchema, ToResponse)]
struct User {
    // Add example value to the Specs and make a type hint 
    #[schema(format="Int64", example=16)]
    id: Option<i64>,
    // Add example value to the Specs and make a type hint#
    #[schema(format="email", example="user@domain.tld")]
    email: String,

    #[serde(with = "time::serde::iso8601", default = "OffsetDateTime::now_utc")]
    #[schema(format="DateTime")]
    created_at: OffsetDateTime,

    #[serde(default="bool::default")]
    admin: bool,
}
```

Adding the controller (and publish the specs) to the server is then simple:

```rust
#[tokio::main]
async fn main() {
    Server::new(vec![
        rwf::controller::OpenApiController.route("/openapi"),
        UsersController::default().crud("/api/users"),
    ])
    .launch()
    .expect("failed to shut down server");
}
```

The `crud` method will automatically implement the following routes:

| Path | Method | Description                                                                                            |
|------|--------|--------------------------------------------------------------------------------------------------------|
| `/api/users` | GET    | List all users. Supports pagination, e.g. `?page_size=25&page=1`. Default page size is 25.             |
| `/api/users/:id` | GET    | Fetch a user by primary key.                                                                           |
| `/api/users`| POST   | Create a new user. All fields not marked optional or not having serde-specified defaults are required. |
| `/api/users/:id` | PUT    | Update a user. Same requirement for fields as the create method above.                                 |
| `/api/users/:id` | PATCH  | Update a user. Only the fields that have changed can be supplied.                                      |
| `/api/users/:id` | DELETE | Delete a user by primary key                                                                           |


### Testing

You can test this with cURL (or your favorite API test client, e.g. Postman):

```
$ curl localhost:8000/api/users -d '{"email": "test@test.com"}' -w '\n'
{"id":1, email":"test@test.com","created_at":"+002024-10-09T22:59:10.693321000Z","admin":false}
$ curl localhost:8000/openapi/yaml 
APISPECS
```