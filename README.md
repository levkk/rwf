# Rum &dash; Rust Web Framework

Rum is a comprehensive framework for building web applications in Rust. Written using the classic MVC  pattern (model-view-controller), Rum comes standard with everything you need to easily build fast and secure web apps.

### Features overview

- :heavy_check_mark: [HTTP server](examples/quick-start)
- :heavy_check_mark: User-friendly [ORM](examples/orm) to build PostgreSQL queries easily
- :heavy_check_mark: [Dynamic templates](examples/dynamic-templates)
- :heavy_check_mark: [Authentication](examples/auth) & built-in user sessions
- :heavy_check_mark: [Middleware](examples/middleware)
- :heavy_check_mark: [Background jobs](examples/background-jobs) and [scheduled jobs](examples/scheduled-jobs)
- :heavy_check_mark: Database migrations
- :heavy_check_mark: Built-in [RESTful framework](examples/crud) with JSON serialization
- :heavy_check_mark: WebSockets support
- :heavy_check_mark: [Static files](examples/static-files) hosting
- :heavy_check_mark: Tight integration with [Hotwired Turbo](https://turbo.hotwired.dev/) for building [backend-driven SPAs](examples/turbo) 
- :heavy_check_mark: Environment-specific configuration
- :heavy_check_mark: Logging and metrics

## Quick start

To add Rum to your stack, create a Rust binary application and add `rum` and `tokio` to your dependencies:

```bash
cargo add --git https://github.com/levkk/rum rum
cargo add tokio@1 --features full
```

Building an app is then as simple as:

```rust
use rum::prelude::*;
use rum::http::Server;

#[derive(Default)]
struct IndexController;

#[rum::async_trait]
impl Controller for IndexController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::new().html("<h1>Hey Rum!</h1>"))
    }
}

#[tokio::main]
async fn main() {
    Server::new(vec![
        IndexController::default().route("/"),
    ])
    .launch("0.0.0.0:8000")
    .await
    .expect("error shutting down server");
}
```

### Examples

See [examples](examples) for common use cases.

## Status :construction:

Rum is in early development and not ready for production. Contributions are welcome. Please see [CONTRIBUTING](CONTRIBUTING.md) for guidelines, [ARCHITECTURE](ARCHITECTURE.md) for a tour of the code, and [ROADMAP](ROADMAP.md) for a non-exhaustive list of desired features.

## Features

### HTTP server

Rum's built-in HTTP server is asynchronous and supports millions of connections.

### The ORM

Rum's ORM is inspired by a healthy mix of Django and ActiveRecord. Declaring models is as simple as:

```rust
use rum::prelude::*;
use time::OffsetDateTime;

#[derive(rum::macros::Model)]
struct User {
    id: Option<i64>,
    email: String,
    created_at: OffsetDateTime,
    admin: bool,
}
```

#### Creating records

Creating new records can be done in two ways: by saving a record with no primary key or by explicitly using `Model::create`.

##### Record with no primary key

```rust
let user = User {
    id: None,
    email: "hello@test.com".into(),
    created_at: OffsetDateTime::now_utc(),
    admin: false,
};

let user = user
    .save()
    .fetch(&mut conn)
    .await?;
```

##### Creating explicitly

```rust
let user = User::create(&[
    ("email", "hello@test.com".to_value()),
    ("created_at", OffsetDateTime::now_utc().to_value()),
    ("admin", false.to_value())
])
    .fetch(&mut conn)
    .await?;
```

If your database schema has default values for columns, you don't have to specify them when creating records, for example:

```rust
let user = User::create(&[
    ("email", "hello@test.com"),
])
    .fetch(&mut conn)
    .await?;
```

##### Handling conflicts

If you are not sure if the record already exists, you can find it first, and if it doesn't exist, create it automatically:

```rust
let user = User::create(&[
    ("email", "hello@test.com"),
])
    .unique_by(&["email"])
    .find_or_create()
    .fetch(&mut conn)
    .await?;
```

This will issue up to two queries:

1. `SELECT` to find the record, and if it doesn't exist
2. `INSERT ... ON CONFLICT DO UPDATE` to insert a new record, and if a conflict is found, it will be resolved without returning errors

#### Finding records

Rum's ORM supports many ways for fetching records, including joins, OR-queries, and `SELECT FOR UPDATE` for exclusive locks.

##### Find by primary key

Finding a record by primary key is as simple as:

```rust
let user = User::find(15)
    .fetch(&mut conn).await?;
```

If the record with `id = 15` does not exist, an error will be returned. To avoid getting an error, use `fetch_optional` or `fetch_all` instead:

```rust
let user = User::find(15)
    .fetch_optional(&mut conn).await?;
```

##### Searching by fields

Filtering on one or multiple fields is easy:

```rust
use time::Duration;

let new_admins = User::all()
    .filter("admin", true)
    .filter_gte("created_at", OffsetDateTime::now_utc() - Duration::days(1))
    .fetch_all(&mut conn)
    .await?;
```

Basic comparison operations are supported:

| Operation | Function |
|-----------|----------|
| `=` | `filter` |
| `<` | `filter_lt` |
| `>` | `filter_gt` |
| `<=` | `filter_lte` |
| `>=` | `filter_gte` |
| `!=` | `not` / `filter_not` |
| `IN` | `filter` with a slice as the value |
| `NOT IN` | `not` / `filter_not` with a slice as the value |

For example, finding records with specific emails:

```rust
User::filter("email", ["joe@hello.com", "marry@hello.com"].as_slice())
    .fetch_all(&mut conn)
    .await?;
```

#### Scopes

If a query is used frequently, you can add it as as scope to the model:

```rust
impl User {
    /// Get all admin users.
    pub fn admins() -> Scope<User> {
        User::all()
            .filter("admin", true)
    }
}

let admins = User::admins()
    .fetch_all(&mut conn)
    .await?;
```

Scopes can be chained to write complex queries easily:

```rust
impl User {
    /// Get users created recently.
    pub fn created_recently(scope: Scope<User>) -> Scope<User> {
        scope.filter_gte(
            "created_at",
            OffsetDateTime::now_utc() - Duration::days(1)
        )
    }

    /// Get admins created recently.
    pub fn new_admins() -> Scope<User> {
        User::created_recently(User::admins())
    }
}
```

#### Updating records

Updating records can be done in two ways: by saving an existing record or by using `update_all` on a scope.

##### Updating existing records

Updating an existing record can be done by mutating fields on a record and calling `save`:

```rust
let mut user = User::find(15)
    .fetch(&mut conn)
    .await?;

user.admin = true;

let admin = user
    .save()
    .fetch(&mut conn)
    .await?;
```

#### Updating many records

```rust
// Remove superpowers from everyone.
User::filter("admin", true)
    .update_all(&[
        ("admin", false)
    ])
    .execute(&mut conn)
    .await?;
```