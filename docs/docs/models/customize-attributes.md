# Customize attributes

When defining models, the `Model` macro makes certain assumptions about your database table and column names. For example, the name of the table is derived from the struct name:

```rust
#[derive(Clone, macros::Model)]
struct User {
    id: Option<i64>,
}
```

The name of the struct, `User` is lowercased and pluralized, to derive the table name `"users"`. Similarly, the foreign key for the `"users"` table is derived to be `"user_id"`.

It's possible to override this behavior, by specifying both table name and foreign key names manually:

```rust
#[derive(Clone, macros::Model)]
#[table_name("my_user_table")]
#[foreign_key("u_id")]
struct User {
    id: Option<i64>,
}
```
