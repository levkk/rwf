# Debug queries

When building queries using the ORM, the end result can be inspected by calling `to_sql()`:

=== "Rust"
    ```rust
    let query = User::all()
      .filter("created_at", Value::Null)
      .limit(5)
      .to_sql();
    println!("Query: {}", query);
    ```
=== "Output"
    ```
    Query: SELECT * FROM "users" WHERE "created_at" IS NULL LIMIT 5
    ```

The query will not be sent to the database, so it's safe to inspect all queries, no matter if they are performant or not.

## Query plan

Visual inspection of the query is often not sufficient to understand query performance. For this purpose, databases like PostgreSQL provide
the `EXPLAIN` functionality which, instead of executing the query, produces an execution plan:

=== "Rust"
    ```rust
    let plan = User::find(15)
      .explain(&mut conn)
      .await?;
    println!("{}", plan);
    ```
=== "SQL"
    ```postgresql
    EXPLAIN SELECT * FROM "users" WHERE "id" = $1
    ```
=== "Output"
    ```
    Seq Scan on users  (cost=0.00..25.00 rows=6 width=40)
    Filter: (id = 5)
    ```

When optimizing queries, this functionality is useful for finding queries that should be using indexes but perform a sequential scan instead.
