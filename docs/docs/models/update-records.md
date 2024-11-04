# Update records

Rwf allows to update records using two mechanisms:

- Update a single record by calling `Model::save` on an instance of a model
- Update multiple records using one query by using `Model::update_all`

## Update a single record

Updating a single record can be done by mutating struct instance fields and calling `save`:

=== "Rust"
    ```rust
    // Get an instance of a User model
    let user = User::all()
      .take_one()
      .fetch(&mut conn)
      .await?;

    // Change a field
    user.created_at = OffsetDateTime::now_utc();

    // Update the record
    let user = user
      .save()
      .fetch(&mut conn)
      .await?;
    ```
=== "SQL"
    ```postgresql
    UPDATE "users" SET "email" = $1, "created_at" = $2 WHERE "id" = $3 RETURNING *
    ```

Instead of fetching the record from the database, you can just instantiate one manually, as long as you know
the desired primary key value:

```rust
let user = User {
  id: Some(25),
  email: "new_email@example.com",
  created_at: OffsetDateTime::now_utc(),
};

let user = user
  .save()
  .fetch(&mut conn)
  .await?;
```

This is very similar to [creating new records](create-records.md), except that we set the `id` field to a known value.
When the `id` is set to `Some(i64)`, Rwf assumes the record exists in the database, meanwhile if the `id` is `None`, Rwf will attempt to create one instead.

## Update multiple records

Updating multiple records in one query is possible by searching for them first and then calling `update_all`:

=== "Rust"
    ```rust
    let users = User::all()
      .filter_gte("created_at", OffsetDateTime::now_utc() - Duration::hours(1))
      .update_all(&[
        ("created_at", OfssetDateTime::now_utc()),
      ])
      .fetch_all(&mut conn)
      .await?;
    ```
=== "SQL"
    ```postgresql
    UPDATE "users" SET created_at = $1 WHERE created_at >= $2
    ```
