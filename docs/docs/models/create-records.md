# Create records

Rwf can create model records in one of two ways:

- `Model::save` method which is called on an instance of a struct implementing the `Model` trait
- `Model::create` method which accepts the column names and their respective values as input

## Saving models

Using our `User` model from our [previous example](index.md), we can create a new record by instantiating a new instance of the `User` struct and calling `save`:

```rust
let user = User {
    id: None,
    email: "admin@example.com".to_string(),
    created_at: OffsetDateTime::now_utc(),
};

let user = user
    .save()
    .fetch(&mut conn)
    .await?;
```

!!! note
    The `id` field is set to `None`. This ensures that the database
    assigns it a value automatically, and that this value is unique.

Calling `save` on a model struct with the `id` set to `None` produces the following query:

```postgresql
INSERT INTO "users" ("email", "created_at") VALUES ($1, $2) RETURNING *
```

## Using table defaults

If you don't want to specify some columns when creating records and your database schema has configured defaults, you can use the `Model::create`
method instead:

=== "Rust"

    ```rust
    let user = User::create(&[
        ("email", "admin@example.com"),
    ])
    .fetch(&mut conn)
    .await?
    ```

=== "SQL"

    ```postgresql
    INSERT INTO "users" ("email") VALUES ($1) RETURNING *
    ```

Any columns not specified in the `INSERT` statement will be automatically filled in with column defaults. For example, the `created_at` column
specified in our [previous example](index.md) has a default value `NOW()`, the current database time.

## Mixing data types

When using `Model::create`, Rwf automatically converts values from Rust to database types. Due to how Rust works, it's not possible to build slices containing values of different types. If you try, you will get `error[E0308]: mismatched types`. To get around this, you can call [`ToValue::to_value`](https://docs.rs/rwf/latest/rwf/model/value/trait.ToValue.html#tymethod.to_value) on each column, for example:

=== "Rust"
    ```rust
    let user = User::create(&[
        ("email", "user@example.com".to_value()),
        ("created_at", OffsetDateTime::now_utc().to_value()),
    ])
    .fetch(&mut conn)
    .await?;
    ```
=== "SQL"
    ```postgresql
    INSERT INTO "users" ("email", "created_at") VALUES ($1, $2) RETURNING *
    ```

## Unique constraints

It's very common to place unique constraints on certain columns in a table to avoid duplicate records. For example, the `"users"` table
would typically have a unique constraint on the `email` column, ensuring that no two users have the same email address.

To handle unique constraints, Rwf can update a record in-place if one exists already matching the constraint:

=== "Rust"
    ```rust
    let user = User::create(&[
      ("email", "admin@example.com")
    ])
    .unique_by(&["email"])
    .fetch(&mut conn)
    .await?;
    ```
=== "SQL"
    ```postgresql
    INSERT INTO "users" ("email") VALUES ($1)
    ON CONFLICT ("email") DO UPDATE
    SET "email" = EXCLUDED."email"
    RETURNING *
    ```

## Optionally create records

If the record matching the `INSERT` statement exists already, Rwf supports returning the existing row without performing an update:

=== "Rust"
    ```rust
    let user = User::find_or_create_by(&[
      ("email", "user1@example.com")
    ])
    .fetch(&mut conn)
    .await?;
    ```
=== "SQL"
    ```postgresql
    SELECT * FROM "users" WHERE "email" = $1;
    INSERT INTO "users" ("email") VALUES ($1) RETURNING *;
    ```

This executes _up to_ two queries, starting with a `SELECT` to see if a row already exists, and if it doesn't, an `INSERT` to create it.

### Combining with a unique constraint

In busy web apps which execute thousands of queries per second, it's entirely possible for a record to be created between the time the `SELECT` query
returns no rows and an `INSERT` query is sent to the database. In this case, a unique constraint violation error will be returned. To avoid this,
it's possible to combine `unque_by` with `find_or_create_by` executed inside a single transaction:

=== "Rust"
    ```rust
    // Start a transaction explicitely.
    let transaction = Pool::transaction().await?;

    let user = User::find_or_create_by(&[
      ("email", "user1@example.com")
    ])
    .unique_by(&["email"])
    .fetch(&mut transaction)
    .await?;

    // Commit the transaction.
    transaction.commit().await?;
    ```
=== "SQL"
    A transaction is started explicitly:
    ```postgresql
    BEGIN
    ```

    Afterwards, the ORM attempts to find a record matching the columns
    in the `INSERT` statement:

    ```postgresql
    SELECT * FROM "users" WHERE "email" = $1
    ```

    If this query returns a row, no more queries are executed. Otherwise,
    an `INSERT` query with `ON CONFLICT` clause is sent to the database:

    ```postgresql
    INSERT INTO "users" ("email") VALUES ($1)
    ON CONFLICT ("email") DO UPDATE
    SET "email" = EXCLUDED."email"
    RETURNING *
    ```

    Finally, the transaction is committed to the database:

    ```postgresql
    COMMIT
    ```
