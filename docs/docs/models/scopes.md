# Scopes

A scope, in ORM language, is a commonly used query which can be saved and re-used in different parts of the code without having
to type it out multiple times.

## Declare scopes

Rwf provides the `Scope<Model>` generic type which indicates that the value is a non-executed query. It's possible to declare scopes directly on the
model struct (or as a separate function anywhere else), for example:

```rust
impl User {
    fn admins() -> Scope<User> {
        User::all()
            .filter("email", &[
                "admin@example.com",
                "boss@example.com",
                "joe@example.com",
            ])
    }
}
```

The query is now re-usable anywhere in the code base:

```rust
let admins = User::admins()
    .fetch_all(&mut conn)
    .await?;
```

!!! note
    When defining scopes, it's important _not_ to execute the scope before returning it. In the example above,
    you'll note that we don't call `fetch`, or `fetch_all` but return the result of calling `filter` instead.


## Chain scopes

It's possible to build very complex queries easily, by chaining multiple scopes together:

```rust
impl User {
    /// Admins created in the last hour.
    fn new_admins() -> Scope<User> {
        User::admins()
            .filter_gte(
                "created_at",
                OffsetDateTime::now_utc() - Duration::hours(1),
            )
    }

    /// New admins ordered by primary key.
    fn new_admins_ordered() -> Scope<User> {
        User::new_admins()
            .order("id")
    }
}
```

## Scopes and joins

It's entirely possible to save complex joins in a scope, for example:

```rust
impl User {
    /// Users who understand how to write Rust macros.
    fn intermediate() -> Scope<User> {
        User::all()
          .join::<Project>
          .filter("project_name", "How to write Rust macros")
          .filter("completed", true)
    }
}
```
