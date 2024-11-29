# Model controller

It's common for applications that use [REST](index.md) with the [ORM](../../models/index.md) to translate the six REST verbs into four data operations, called CRUD: **C**reate. **R**ead, **U**pdate, and **D**elete. This allows frontend code to directly manipulate backend models, shifting most of the application logic to JavaScript frameworks like React or Vue.

To avoid writing boilerplate code, Rwf provides the [`ModelController`](https://docs.rs/rwf/latest/rwf/controller/trait.ModelController.html) which matches the REST verbs to CRUD operations automatically:

```rust
use rwf::prelude::*;
use serde::{Serialize, Deserialize};

/// The model.
#[derive(Clone, macros::Model, Serialize, Deserialize)]
struct User {
    id: Option<i64>,
    email: String,
}

/// The controller.
#[derive(Default, macros::ModelController)]
struct Users;

#[async_trait]
impl ModelController for Users {
    type Model = User;

    /* All methods are implemented automatically. */
}
```

## Connecting to a route

To add a CRUD controller to the server, you can use the `crud!` macro, for example:

```rust
use rwf::prelude::*;
use rwf::http::Server;

#[tokio::main]
async fn main() {
    Server::new(vec![
        crud!("/users" => Users),
    ])
    .launch()
    .await
    .unwrap()
}
```

!!! note
    Just like the `rest!` and `route!` macros, the `crud!` macro is optional. It translates to `Users::default().crud("/users")`
    which can be written manually if special initialization for the controller is required.

## Using the controller

Frontend code can directly fetch, create and modify resources using the controller, for example:

=== "List users"
    ```javascript
    let response = await fetch("/users");
    const users = await response.json();

    console.log(users)
    ```
=== "Output"
    ```json
    [
      {"id": 1, "email": "admin@example.com", "admin": true},
      {"id": 2, "email": "alice1@example.com", "admin": false}
    ]
    ```

=== "Update user"
    ```javascript
    let response = await fetch("/users/2", {
      method: "PATCH",
      body: JSON.stringify({admin: true}),
    });

    const user = await response.json();

    console.log(user)
    ```
=== "Output"
    ```json
    {"id": 2, "email": "alice1@example.com", "admin": true}
    ```

### Pagination

To avoid excessive data transfer and slow database queries, the model controller uses pagination on the list endpoint. Resources are returned in pages of 25 items each. You can paginate between them by passing the `page` query parameter, for example:

```
GET /users?page=5
```

To control the page size, pass the `page_size` query parameter, for example:

```
GET /users?page=1&page_size=50
```

## JSON serialization

The model controller uses JSON serialization powered by the [`serde_json`](https://docs.rs/serde_json) crate. When implementing the [`ModelController`](https://docs.rs/rwf/latest/rwf/controller/trait.ModelController.html) for a model, make sure to derive the `Serialize` and `Deserialize` traits.

[`serde`](https://docs.rs/serde) is very flexible and allows you to control every aspect of serialization and deserialization. You can rename, hide, overwrite, and ignore any model fields. See [Field attributes](https://serde.rs/field-attrs.html) for more information on how to customize JSON (de)serialization.

## Learn more

- [examples/rest](https://github.com/levkk/rwf/tree/main/examples/rest)
- [Serde field attributes](https://serde.rs/field-attrs.html)
