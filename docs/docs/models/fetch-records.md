# Fetch records

## Retrieve by primary key

Retrieving a record by primary key method accepts an integer and returns a single row corresponding to where the value of the `id` column equals to the integer:

=== "Rust"

    ```rust
    let user = User::find(15)
        .fetch(&mut conn)
        .await?

    assert_eq!(user.id, 15);
    ```

=== "SQL"

    ```postgresql
    SELECT * FROM "users" WHERE "id" = $1
    ```

## Searching records

Rwf supports searching records by any column in its respective table. For example, the `User` model has three columns, all of which are searchable:

=== "Rust"

    ```rust
    use time::Duration;

    let users = User::all()
      .filter("email", "admin@example.com")
      .filter_gte("created_at", OffsetDateTime::now_utc() - Duration::hours(1))
      .fetch_all(&mut conn)
      .await?

    assert_eq!(users.len(), 1);
    ```

=== "SQL"

    ```postgresql
    SELECT * FROM "users" WHERE "email" = $1 AND "created_at" >= $2
    ```

=== "Placeholders"

    | ID | Value |
    |----|-------|
    | `$1` | `'admin@example.com'` |
    | `$2` | `'2024-10-16T11:27:56-07:00'` |


Rwf supports multiple comparison operations for most data types:

| Method | Operator | Example |
|--------|----------|-----------|
| `filter` | `=` | `id = 5` |
| `not` / `filter_not` | `!=` | `email != 'user@example.com'` |
| `filter_gt` | `>` | `created_at > NOW()` |
| `filter_lt` | `<` | `created_at < '2024-10-16'` |
| `filter_gte` | `>=` | `id >= 5` |
| `filter_lte` | `<=` | `id <= 25` |
| `filter` | `IN` | `id IN (1, 2, 3)` |
| `not` | `NOT IN` | `id NOT IN (4, 5, 6)` |

The `filter` (and `not`) methods accept lists of values (in Rust, those are called "slices") which translate to the `IN` and `NOT IN` filters in SQL respectively:

=== "Rust"
    ```rust
    let users = User::all()
      .filter("email", &["user1@example.com", "user2@example.com"])
      .fetch_all(&mut conn)
      .await?;

    assert_eq(users.len(), 2);
    ```
=== "SQL"
    ```postgresql
    SELECT * FROM "users" WHERE "email" = ANY($1)
    ```

    !!! note
        `= ANY('{1, 2, 3}')` is equivalent to `IN (1, 2, 3)`. In fact, when performing an index scan
        using an `IN` (or `NOT IN`) clause, the query is translated by the database to use `ANY` instead.

### Search by `NULL`

Searching columns that have no value, i.e. the value is `NULL`, is a special case and is handled by passing the `Value::Null` explicitly:

=== "Rust"
    ```rust
    let users = User::all()
      .filter("email", Value::Null)
      .fetch_all(&mut conn)
      .await?;

    assert_eq!(users.len(), 0);
    ```

=== "SQL"
    ```postgresql
    SELECT * FROM "users" WHERE "email" IS NULL
    ```

Searching by the opposite, where a column is not `NULL`:

=== "Rust"
    ```rust
    let users = User::all()
      .not("email", Value::Null)
      .count(&mut conn)
      .await?;
    ```

=== "SQL"
    ```postgresql
    SELECT COUNT(*) FROM "users" WHERE email IS NOT NULL
    ```

### Optional results

When using `fetch`, if no rows exist, the ORM will return a `RecordNotFound` error.
To avoid this, use `fetch_optional` which will return an `Option` instead:

=== "Rust"
    ```rust
    let user = User::all()
      .take_one()
      .fetch_optional(&mut conn)
      .await?;

    assert!(user.is_some());
    ```
=== "SQL"
    ```postgresql
    SELECT * FROM "users" LIMIT 1
    ```

## Limiting results

Fetching many records at once can be inefficient and slow. To limit how many rows your queries return, you can add a `LIMIT` clause:

=== "Rust"
    ```rust
    let first_25 = User::all()
      .limit(25)
      .order("id")
      .fetch_all(&mut conn)
      .await?;

    assert_eq!(first_25.len(), 25);
    ```
=== "SQL"
    ```postgresql
    SELECT * FROM "users" ORDER BY "id" LIMIT 25
    ```

### Paginating results

Pagination is supported using the `OFFSET` clause:

=== "Rust"
    ```rust
    let next_25 = User::all()
      .limit(25)
      .offset(25)
      .order("id")
      .fetch_all(&mut conn)
      .await?;
    ```
=== "SQL"
    ```postgresql
    SELECT * FROM "users" ORDER BY "id" LIMIT 25 OFFSET 25
    ```

## Ordering results

It's often more efficient and simpler to order rows in the database instead of in the application. Rwf supports ordering by any column
in the query, by specifying them using the `order` method:

=== "Rust"
    ```rust
    let users = User::all()
      .order("email")
      .order(("id", "DESC"))
      .fetch_all(&mut conn)
      .await?;
    ```
=== "SQL"
    ```postgresql
    SELECT * FROM "users" ORDER BY "email", "id" DESC
    ```

## Locking rows

In busy production applications, it's common for the same row to be accessed from multiple places at the same time. If you'd like to prevent that row from being
accessed while you're doing something to it, for example updating it with new values, you can use a row-level lock:

=== "Rust"
    ```rust
    let transaction = Pool::transaction().await?;

    let user = User::find(15)
        .lock()
        .fetch(&mut transaction)
        .await?;

    transaction.commit().await?;
    ```
=== "SQL"
    ```postgresql
    BEGIN;
    SELECT * FRON "users" WHERE "id" = $1 FOR UPDATE;
    COMMIT;
    ```


The lock on the row(s) returned by a query last only for the duration of the transaction. It's common to use that time to update multiple tables that have some kind of
relationship to the row being locked. This mechanism allows to perform atomic operations (all or nothing) in a concurrent environment without data races or inconsistencies.
