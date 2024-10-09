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

## :construction: Status :construction:

Rum is in early development and not ready for production. Contributions are welcome. Please see [CONTRIBUTING](CONTRIBUTING.md) for guidelines, [ARCHITECTURE](ARCHITECTURE.md) for a tour of the code, and [ROADMAP](ROADMAP.md) for a non-exhaustive list of desired features.

## Features

### HTTP server

Rum's built-in HTTP server is asynchronous and supports millions of connections.

### The ORM

Rum's ORM is inspired by a healthy mix of Django and ActiveRecord. Declaring models is as simple as:

```rust
use rum::prelude::*;
use time::OffsetDateTime;

#[derive(Clone, rum::macros::Model)]
struct User {
    id: Option<i64>,
    email: String,
    created_at: OffsetDateTime,
    admin: bool,
}
```

#### Creating records

Creating new records can be done in two ways: by saving a record with an empty primary key or by explicitly using `Model::create` method.

##### Record with empty primary key

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

##### Creating records explicitly

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

##### Rust to Postgres type conversion

Rust types are converted to Postgres values automatically. If multiple Rust types are used in a single value, e.g. a slice, which the Rust compiler does not allow, the values can be converted to an internal representation, `rum::model::Value`, explicitly by calling `ToValue::to_value`:

```rust
let pg_value = 1_i64.to_value();
```

##### Handling conflicts

If your table has a unique index, you may run into unique constraint violations when creating records. To avoid that, you can use PostgreSQL's `ON CONFLICT DO UPDATE` feature, which Rum's ORM supports out of the box:

```rust
let user = User::create(&[
    ("email", "hello@test.com"),
])
    .unique_by(&["email"])
    .fetch(&mut conn)
    .await?;
```

If you are reasonably confident the record already exists, you can avoid writing to the table by searching for it first:

```rust
let user = User::find_or_create_by(&[
        ("email", "hello@test.com"),
    ])
    .unique_by(&["email"])
    .fetch(&mut conn)
    .await?;
```

This will execute up to two queries:

1. `SELECT` to find the record, and if it doesn't exist
2. `INSERT ... ON CONFLICT DO UPDATE` to insert a new record, updating it in-place if it exists

If the table doesn't have unique constraints, you can still use `find_or_create_by`, except duplicate records can be created if the same query is executed more than once:

```rust
let user = User::find_or_create_by(&[("email", "hello@test.com")])
    .fetch(&mut conn)
    .await?;
```

#### Finding records

Rum's ORM supports many ways for fetching records, including searching by any column, joining tables, OR-ing multiple conditions together, and row-level locking.

##### Find by primary key

Find a record by primary key:

```rust
let user = User::find(15)
    .fetch(&mut conn).await?;
```

If the record with `id = 15` does not exist, an error will be returned. To avoid getting an error, use `fetch_optional` or `fetch_all` instead:

```rust
let user = User::find(15)
    .fetch_optional(&mut conn).await?;
```

This executes the following query:

```sql
SELECT * FROM users WHERE id = 15;
```

##### Searching by fields

Filtering on one or multiple fields:

```rust
use time::Duration;

let new_admins = User::all()
    .filter("admin", true)
    .filter_gte("created_at", OffsetDateTime::now_utc() - Duration::days(1))
    .fetch_all(&mut conn)
    .await?;
```

Basic comparison operations on most data types are supported:

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

Multiple records can be updated without fetching them from the database:

```rust
// Remove superpowers from everyone.
User::filter("admin", true)
    .update_all(&[
        ("admin", false)
    ])
    .execute(&mut conn)
    .await?;
```

This executes only one query, updating records matching the filter condition.

#### Joins



```rust
#[derive(Clone, rum::macros::Model)]
#[belongs_to(User)]
struct Order {
    id: Option<i64>,
    user_id: i64,
    total_amount: f64,
    refunded_at: Option<OffsetDateTime>,
}

#[derive(Clone, rum::macros::Model)]
#[belongs_to(Order)]
struct Product {
    id: Option<i64>,
    order_id: i64,
    name: String,
    price: f64,
}
```

Searching for records can now be done by joining two (or more) tables together:

```rust
// Find users that paid us at least $1.
let paying_users = User::all()
    .join::<Order>()
    .filter_gte(Order::column("total_amount"), 1.0)
    .fetch_all(&mut conn)
    .await?;
```
