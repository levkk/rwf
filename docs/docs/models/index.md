# ORM basics

## Introduction

Rwf comes with its own [ORM](https://en.wikipedia.org/wiki/Object%E2%80%93relational_mapping) (object-relational mapping). The Rwf ORM is
very flexible, supporting anything from basic fetch by primary key queries, to multi-table joins and complex custom queries.

### What's an ORM?
The ORM is the **M** in MVC design: the model. It allows to easily retrieve data stored in your database tables
and display it in the application, without having to write complex SQL queries by hand.

It works by attaching itself to Rust structs and mapping data from table columns to struct fields (and vice versa),
converting them from database types to Rust data types automatically in the process.

## Getting started

Using the ORM is simple and only requires defining a struct for each model (or database table). For example, most web apps will have a `User` model,
which stores its data in a `"users"` table:

| Column | Database data type | Rust data type |
|--------|-----------|---------------|
| `id` | `BIGINT` | `i64` |
| `email` | `VARCHAR` | `String` |
| `created_at` | `TIMESTAMPTZ` | `time::OffsetDateTime` |

Defining the Rust struct for the model can be done as follows:

```rust
use rwf::prelude::*;

#[derive(Clone, macros::Model)]
struct User {
    id: Option<i64>,
    email: String,
    created_at: OffsetDateTime,
}
```

The same table in the database can be created with this query[^1]:

```postgresql
CREATE TABLE users (
  id BIGSERIAL PRIMARY KEY,
  email VARCHAR NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

!!! note
    The `id` column is using an optional Rust `i64` integer. This is because the struct will be used
    for both inserting and selecting data from the table. When inserting, the `id` column should be `None` and will be automatically
    assigned by the database. This ensures that all rows in your tables have a unique primary key.

[^1]: See [migrations](migrations.md) to learn how to create tables in your database reliably.

### Naming conventions
The struct fields have the same name as the database columns, and the data types match their respective Rust types. The table name in the database corresponds to the name of the struct, lowercase and pluralized. For example, `User` model will refer to the `"users"` table in the database.

A row in a database table which contains model data is called a record. The `macros::Model` macro automatically implements the database to Rust and vice versa types conversion
and maps the column values to the struct fields.

## Query data

With the model defined in Rust, writing SQL queries is automatically implemented by the ORM. For example, to fetch a record by primary key,
you can do the following:

```rust
let user = User::find(15)
    .fetch(&mut conn)
    .await?;
```

The `find` method is implemented by the `Model` trait for the `User` struct automatically. It accepts a Rust integer and produces the following query:

```postgresql
SELECT * FROM "users" WHERE id = $1
```

The `fetch` method assembles the query, sends it to the database, and returns one row. The row is converted to an instance of the `User` struct:

```rust
println!("user email: {}", user.email);
```

## Fetch multiple rows

Querying multiple rows can be done by using `fetch_all` instead of `fetch`, for example:

```rust
let users = User::all()
    .order("id")
    .limit(25)
    .fetch_all(&mut conn)
    .await?;
```

This will fetch 25 user records from the `"users"` table, ordering them by the primary key. The result will be a `Vec<User>`, in the order
returned by the database:

```rust
for user in &users {
    println!("{}: {}", user.id, user.email);
}
```

The ORM can be used to write easy and complex queries alike, without having to learn SQL.
Rwf currently supports PostgreSQL, but other databases like SQLite and MySQL are on the roadmap.
