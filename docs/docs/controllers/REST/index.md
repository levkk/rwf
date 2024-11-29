# REST overview

[REST](https://en.wikipedia.org/wiki/REST) (Representational State Transfer) is a design pattern for web applications that separates frontend and backend interactions into six (6) predefined operations, called REST verbs. RESTful apps use those verbs to fetch and change application state.

## Six methods

### List

The list method, implemented using the HTTP `GET /endpoint` call, is meant to produce a list of resources found at a particular endpoint. For example, if you have a `/users` endpoint, executing `GET /users` should return a list of users visible to the user making the request:

=== "Request"
    ```bash
    curl localhost:8000/users
    ```
=== "Response"
    ```json
    [
      {"id": 1, "email": "admin@example.com", "admin": true},
      {"id": 2, "email": "first-user@example.com", "admin": false}
    ]
    ```

### Get

The get method, implemented using the HTTP `GET /endpoint/<id>` call, where `<id>` is a unique resource identifier, e.g., a number or a UUID, fetches one specific resource found at this endpoint and which has the provided identifier. For example, if the client requests the `/users/1` resource, the framework would return one `User` object with the primary key `1`:

=== "Request"
    ```bash
    curl localhost:8000/users/1
    ```
=== "Response"
    ```json
    {
      "id": 1,
      "email": "admin@example.com",
      "admin": false
    }
    ```

### Create

Create, implemented using the HTTP `POST /endpoint` call, creates a new resource at that endpoint. The caller specifies the resource definition, which the server needs to validate, and will return the resource created, along with any server-generated fields like the unique resource identifier.

Taking the `/users` endpoint example, executing `POST /users` would create a new user in the application:

=== "Request"
    ```bash
    curl localhost:8000/users \
      -d '{"email": "alice@example.com", "admin": false}'
    ```
=== "Response"
    ```json
    {
      "id": 45,
      "email": "alice@example.com",
      "admin" false
    }
    ```

### Update

The update method, implemented using the HTTP `PUT /endpoint/<id>` call, updates an existing resource with new values, and returns the updated resource:

=== "Request"
    ```bash
    curl -X PUT localhost:8000/users/45 \
      -d '{"email": "alice1@example.com", "admin": false}'
    ```
=== "Response"
    ```json
    {
      "id": 45,
      "email": "alice1@example.com",
      "admin": false
    }
    ```

### Patch

The patch method, implemented using the HTTP `PATCH /endpoint/<id>` call, performs a partial update of the resource, allowing the caller to send only the fields in the resource object that should be changed:

=== "Request"
    ```bash
    curl -X PATCH localhost:8000/users/45 -d '{"admin": true}'
    ```
=== "Response"
    ```json
    {
      "id": 45,
      "email": "alice1@example.com",
      "admin": true
    }
    ```

### Delete

The delete method, implemented using the HTTP `DELETE /endpoint/<id>` call, deletes a resource:

=== "Request"
    ```bash
    curl -X DELETE localhost:8000/users/45
    ```
=== "Response"
    ```
    (empty)
    ```

## REST controller

Rwf comes with a REST [controller](../index.md), which has the six aforementioned methods separated into individual functions. For example, writing a `/users` endpoint controller could be done like so:

```rust
use rwf::prelude::*;

#[derive(Default, macros::RestController)]
struct Users;

#[async_trait]
impl RestController for Users {
    // Users controller uses the primary key to identify resources.
    type Resource = i64;

    /// List all users.
    /// "GET /users"
    async fn list(&self, request: &Request) -> Result<Response, Error> {
        let users = serde_json::json!([
            {"id": 1, "email": "admin@example.com", "admin": true},
            {"id": 2, "email": "alice@example.com", "admin": false},
        ]);

        Ok(Response::new().json(users)?)
    }

    /// Get a particular user by identifier.
    /// "GET /users/<id>"
    async fn get(&self, request: &Request, id: &i64) -> Result<Response, Error> {
        let user = serde_json::json!({
            "id": *id,
            "email": "admin@example.com",
            "admin": true
        });

        Ok(Response::new().json(user)?)
    }

    /* Optionally implement other REST methods */
}
```

The [`RestController`](https://docs.rs/rwf/latest/rwf/controller/trait.RestController.html) has all six methods (list, get, update, patch, create, delete) and automatically splits the traffic based on the request path and HTTP method used. Implementing any of them is optional. If you don't implement it, the framework will return HTTP 405 `Method Not Allowed`.

### Resource identifier
When implementing the controller, you need to specify the data type used as the identifier for your resources. Example above uses `i64`, which when converted to the database datatype becomes `BIGINT` (or `BIGSERIAL`), but any data types are supported, for example:

```rust
type Resource = String;
```

The identifier data type only needs to implement the [`ToParameter`](https://docs.rs/rwf/latest/rwf/http/path/to_parameter/trait.ToParameter.html) trait.

## Connecting to a route

When launching your server, add your REST controllers to the server routes using the `rest!` macro (instead of the usual `route!`), for example:

```rust
use rwf::prelude::*;
use rwf::http::Server;

#[tokio::main]
async fn main() {
    Server::new(vec![
        rest!("/users" => Users),
    ])
    .launch()
    .await
    .unwrap()
}
```

The `rest!` macro will ensure that all [six](#six-methods) REST-style paths are sent to the `Users` controller.

!!! note
    The `rest!` macro translates to `Users::default().rest("/users")`. The `Users` struct should implement the `Default`
    trait for this to work. You don't have to use the macro and can connect a controller to the server manually.

## Learn more

- [Model controller](model-controller.md)
- [examples/rest](https://github.com/levkk/rwf/tree/main/examples/rest)
