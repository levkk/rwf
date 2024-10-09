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

Rust types are converted to Postgres values automatically. If multiple Rust types are used in a single value, e.g. a slice, which the Rust compiler does not allow, the values can be converted to an internal representation explicitly (`rum::model::Value`), by calling `ToValue::to_value` method:

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

##### Primary key requirement

Unlike ActiveRecord, Rum's ORM requires all models to have a primary key. Without a primary key, operations like joins, updates, and deletes become inefficient and difficult.

Rum currently defaults the the `id` column as the primary key. Customizing the primary key is on the roadmap, including allowing compound keys.

##### Searching by multiple columns

Filtering on one or multiple columns:

```rust
use time::Duration;

let new_admins = User::all()
    .filter("admin", true)
    .filter_gte("created_at", OffsetDateTime::now_utc() - Duration::days(1))
    .filter_lte("created_at", OffsetDateTime::now_utc())
    .fetch_all(&mut conn)
    .await?;
```

which produces the following query:

```sql
SELECT * FROM users WHERE admin = $1 AND created_at >= $2 AND created_at <= $3
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

For example, finding records by filtering on multiple values:

```rust
User::not("email", ["joe@hello.com", "marry@hello.com"].as_slice())
    .fetch_all(&mut conn)
    .await?;
```

which would produce the following query:

```sql
SELECT * FROM users WHERE email NOT IN ('joe@hello.com', 'marry@hello.com');
```

#### Scopes

If a query is used frequently, you can add it as a scope to the model:

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

Every time the scope is used, the same query will be executed. Scopes can be chained to write complex queries easily:

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

// Give superpowers to this user.
user.admin = true;

let admin = user
    .save()
    .fetch(&mut conn)
    .await?;
```

This will produce the following query:

```sql
UPDATE users SET email = $1, created_at = $2, admin = $3 WHERE id = $4
```

updating all columns based on the values in the Rust struct.


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

#### Concurrent updates

If a record is updated simultaneously from multiple places, one update operation may overwrite another. To prevent this, an exclusive lock can be placed on a record:

```rust
let mut transaction = Pool::begin().await?;

let user = User::find(15)
    .lock()
    .fetch(&mut transaction)
    .await?;

user.admin = true;
user.email = "admin@hello.com".into();

let user = user
    .save()
    .fetch(&mut transaction)
    .await?;

transaction.commit().await?;
```

This will use execute the update inside a transaction, while blocking other queries (including `SELECT`s) until the transaction completes.

```sql
BEGIN;
SELECT * FROM users WHERE id = 15 FOR UPDATE;
UPDATE users SET email = 'admin@hello.com', admin = true WHERE id = 15;
COMMIT;
```

#### Joins

Joins in Rum come standard and require a couple annotations on the models to indicate their relationships:

```rust
#[derive(Clone, rum::macros::Model)]
#[has_many(Order)]
struct User {
    id: Option<i64>,
    email: String,
    created_at: OffsetDateTime,
    admin: bool,
}

#[derive(Clone, rum::macros::Model)]
#[belongs_to(User)]
#[has_many(Product)]
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

Since columns in multiple tables can have the same name, e.g. `id`, `name`, etc, Rum can disambiguate them by including the table name in the column selection:

```rust
let column = Order::column("name");
assert_eq!(column.to_sql(), r#""products"."name""#);
```

#### Nested joins

Joins against models not immediately related to a model are possible by using nested joins:

```rust
let users_that_like_apples = User::all()
    .join::<Order>()
    .filter_gte(Order::column("total_amount"), 25.0)
    .join_nested(Order::join::<Product>())
    .filter(Product::column("name"), "apples")
    .fetch_all(&mut conn)
    .await?;
```

This will produce the following query:

```sql
SELECT "users".* FROM "users"
INNER JOIN "orders" ON "orders"."user_id" = "users"."id"
INNER JOIN "products" ON "products"."order_id" = "orders"."id"
WHERE "orders"."total_amount" >= 25.0 AND
"products"."name" = 'apples';
```

##### Ordering & limits

Fetching records in a particular order can be easily done with:

```rust
let ascending = Order::all()
    .order("total_amount") // ORDER BY total_amount
    .fetch_all(&mut conn)
    .await?;

let descending = Order:all()
    .order(("total_amount", "DESC")) // ORDER BY total_amount DESC
    .fetch_all(&mut order)
    .await?;
```

If joining multiple tables, it's best to disambiguate the ordering column, which is often present in all tables, e.g.:

```rust
let users = User::all()
    .join::<Order>()
    .order(("total_amount", "DESC"))
    .order((User::column("created_at"), "DESC"))
    .limit(25)
    .fetch_all(&mut conn)
    .await?;
```

Adding a limit to a query prevents fetching too many records at once. Limiting and paginating results can be done with `LIMIT` & `OFFSET`, for example:

```rust
let users = User::all()
    .order("id")
    .limit(25)
    .offset(25)
    .fetch_all(&mut conn)
    .await?;
```

will produce the following query:

```sql
SELECT * FROM users ORDER BY id LIMIT 25 OFFSET 25
```

##### Counting rows

Counting rows can be done by calling `count` instead of `fetch`, for example:

```rust
let users_count = User::all()
    .filter("email", Value::Null)
    .count(&mut conn)
    .await?;

assert_eq!(users_count, 0);
```

##### Show the queries

If you want to see what queries Rum is building underneath, you can enable query logging in the [configuration](#configuration) or call `to_sql` on the scope to output the query string, for example:

```rust
let query = User::all().to_sql();
assert_eq!(query, "SELECT * FROM \"users\"");
```

##### Explain

Getting the query plan for a query instead of running it can be done by calling `explain` instead of `fetch`:

```rust
let query_plan = User::all()
    .filter_lte("created_at", OffsetDateTime::now_utc())
    .limit(25)
    .explain(&mut conn)
    .await?;

println!("{}", query_plan);
// Filter: (created_at <= '2024-10-09 10:23:31.561024-07'::timestamp with time zone)
```

If explaining update/insert queries, make sure to do so inside a transaction (and rolling it back when done) to avoid writing data to tables.

##### Fetching related models

To avoid N+1 queries, Rum provides a way to fetch related models in a single query, for example:

```rust
let users = User::all()
    .limit(25)
    .fetch_all(&mut conn)
    .await?;

let users_orders = User::related::<Order>(&users)
    .fetch_all(&mut conn)
    .await?;
```

##### SQL injection

Rum uses prepared statements with placeholders and sends the values to the database separately. This prevents most SQL injection attacks. User inputs like column names are escaped, for example:

```rust
User::all()
    .filter("\"; DROP TABLE users;\"", true)
    .execute(&mut conn)
    .await?;
```

will produce a syntax error:

```
ERROR:  column users."; DROP TABLE users;" does not exist
```

##### Bypassing the ORM

Sometimes a query is too complicated to be written with an ORM. Rum provides a simple "break glass" functionality to pass in arbitrary queries and map them to a model:

```rust
let users = User::find_by_sql(
    "SELECT * FROM users WHERE email LIKE 'hello%' AND created_at < $1",
    &[
        OffsetDateTime::now_utc().to_value(),
    ]
)
    .fetch_all(&mut conn)
    .await?;
```

## Dynamic templates

Rum has its own template language, heavily inspired (if not shamelessly copied) from Rails' ERB.

### Quick example

If you've used Rails before, you'll find this syntax familiar:

```erb
<p><%= text %></p>

<ul>
<% for item in list %>
    <li><%= item.upcase %><li>
<% end %>
<ul>

<script>
<%- no_user_inputs_allowed_code %>
</script>

<% if value == "on" %>
    <p>Now you see me</p>
<% else %>
    <p>Now you don't</p>
<% end %>
```

### Operations

Rum's templates syntax is very small and simple:

| Operation | Description |
|----------|-------------|
| `<%` | Code block start. |
| `%>` | Code block end. |
| `<%=` | Print the following expression value (don't forget to close the code block). |
| `<%-` | Print expression without escaping "dangerous" HTML characters. |
| `<% if expression %>` | If block which evaluates the expression for truthiness. |
| `<% elsif expression %>`| Else if block, works just like the if block. |
| `<% else %>` | Else block. |
| `<% for item in list %>` | For loop. |
| `<% end %>` | Indicates the end of an if statement or for loop. |
| `+`, `-`, `*`, `/`, `==`, `%` | Addition, subtraction, multiplication, division, equality, modulo. |

### Rendering templates

Templates can be rendered directly from a Rust string:

```rust
#[derive(rum::macros::Context)]
struct Index {
    first_name: String,
}

let template = Template::from_str("<p>Ahoy there, <%= first_name %>!</p>")?;
let context = Index { first_name: "Josh".into() };

let result = template.render(context.try_into()?)?;

assert_eq!(result, "<p>Ahoy there, Josh!</p>");
```

Templates can be placed in files anywhere the Rust program can access them:

```rust
let template = Template::cached("templates/index.html").await?;
let result = template.render(context.try_into()?)?;
```

Templates don't have to be HTML, and can be used to render any kind of files, e.g. plain text, CSS, JavaScript, etc.

### Passing values to templates

Rum's templates support many data types, e.g. strings, integers, lists, hashes, and even models. For example, a list of users can be passed directly into a template:

```rust
let users = User::all()
    .fetch_all(&mut conn)
    .await?;

let template = Template::from_str(
"<ul>
    <% for user in users %>
        <li><%= user.email %></li>
    <% end %>
</ul>")?;

#[derive(rum::macros::Context)]
struct Context {
    users: Vec<User>,
}

let context = Context { users };

let rendered = template.render(&context.try_into()?)?;
```

### Caching templates

Reading templates from disk is usually quick, but compiling them can take some time. In development, they are compiled every time they are fetched, which allows to iterate on their contents quickly, while in production they are cached in memory for performance.

The caching behavior is controlled via configuration and requires no code modifications:

```toml
[general]
cache_templates = true
```

See [Configuration](#configuration) for more details on how to configure Rum apps.


## Configuration

Configuring Rum apps can be done via environment variables or a TOML configuration file.

### Rum.toml

Rum.toml is a configuration file using the TOML configuration language.

#### Example

```toml
[general]
log_queries = true
cache_templates = false
```