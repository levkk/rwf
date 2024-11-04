# Group by

Group by queries allow you to perform analysis of your data directly inside the database. They typically don't return original records, but some kind of aggregate instead. For example, the query below calculates how many users are creating accounts every hour:

=== "SQL"
    ```postgresql
    SELECT
        COUNT(*) AS count,
        DATE_TRUNC('hour', created_at) AS created_at
    FROM users
    GROUP BY 2
    ORDER BY 2
    ```
=== "Output"
    ```
     count |       created_at
    -------+------------------------
         1 | 2024-11-04 08:00:00-08
         5 | 2024-11-04 09:00:00-08
        17 | 2024-11-04 10:00:00-08
    ```

## Write a group by

Ergonomic support for group by queries in Rwf is still a work in progress, so for now, you'll need to use [custom queries](custom-queries.md).

### Define a struct

Since Rust is a typed language, it would be best to define a struct for your aggregate. Using the example above, we can create a model like so:

```rust
#[derive(Clone, macros::Model)]
struct UsersPerHour {
    count: i64,
    created_at: OffsetDateTime,
}
```

Using the `Model` macro allows this struct use all ORM features, just like regular models. In fact, any query result can be mapped to a model in Rwf, as long as you define a struct for it.

### Calculate aggregate

Calculating the aggregate using the database can be done with `Model::find_by_sql`, for example:

```rust
let stats = UsersPerHour::find_by_sql("
    SELECT
        COUNT(*) AS count,
        DATE_TRUNC('hour', created_at) AS created_at
    FROM users
    GROUP BY 2
    ORDER BY 2
", &[])
.fetch_all(&mut conn)
.await?;
```

Just like with [custom queries](custom-queries.md), make sure the query returns all columns specified by the struct, with the correct data types.

## Learn more

- [Group by in rwf-admin](https://github.com/levkk/rwf/blob/main/rwf-admin/src/models/mod.rs)
