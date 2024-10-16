# Join models

Joining models together allows to build complex search queries spanning multiple database tables. Rwf supports joining
models out of the box, but requires a couple of annotations to declare relationships between models.

## Define model relationship

Using the `User` model from our [previous example](../), let's define a `Project` model, which will record projects created by the users of our
fictional web app:

```rust
#[derive(Clone, macros::Model)]
#[belongs_to(User)]
struct Project {
    id: Option<i64>,
    user_id: i64,
    project_name: f64,
    completed: bool,
}
```

A few things to unpack here. First, a new macro annotation `belongs_to` indicates the type of relationship this model has to the `User` model.
In this case, it indicates a "belongs to" relationship, meaning each instance of the `Project` model will have one `User` associated to it.

If we were to create a table for storing records of this model, the query could be:

```postgresql
CREATE TABLE projects (
  id BIGSERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL REFERENCES users(id),
  project_name VARCHAR NOT NULL,
  completed BOOLEAN NOT NULL
);
```

### Naming convention

The naming convention for foreign keys is the singular form of the table name its referring to, e.g. `users` becomes `user`, joined with the name of the primary key,
in our case, the `id` column, producing the `user_id` foreign key.

## Join tables

Specifying the `belongs_to` relationship allows us to query the `Project` model and join it to the `User` model easily:

=== "Rust"
    ```rust
    let projects = Project::all()
      .join::<User>()
      .filter("project_name", "My first Rwf web app")
      .filter("email", "user@example.com")
      .fetch_all(&mut conn)
      .await?;
    ```
=== "SQL"
    ```postgresql
    SELECT "projects".* FROM "projects"
    INNER JOIN "users" ON "projects"."user_id" = "users"."id"
    WHERE "project_name" = $1 AND "email" = $2
    ```

!!! note
    The `join::<Model>` method accepts a generic argument specifying which model we are joining to. If the association between `Project` and `User` doesn't
    exist, the Rust compiler will return an error. This helps us avoid common errors by accidently joining tables that don't have a relationship.

## Disambiguating fields

More often than not, two tables have columns with the same name. The most obvious example of this is the primary key, the `id` column by default, which
exists in all Rwf models. To specify which table & column a query is referring to, Rwf provides the ability to fully qualify the column with the table name:

=== "Rust"
    ```rust
    let projects = Project::all()
      .join::<User>()
      .filter(User::column("id"), 5)
      .take_one()
      .fetch(&mut conn)
      .await?;
    ```
=== "SQL"
    ```postgresql
    SELECT * FROM "projects"
    INNER JOIN "users" ON "projects"."user_id" = "users"."id"
    WHERE "users"."id" = $1
    LIMIT 1
    ```


## Inverse relationship

The `Project` model defines a `belongs_to` relationship to the `User` model, but the `User` model doesn't define one to the `Project` model. If we
attempt to join `"users"` to `"projects"` (instead of the other way around), we will get a Rust compiler error. To avoid this, we can specify
the inverse relationship on the `User` model, like so:

```rust
#[derive(Clone, macros::Model)]
#[has_many(Project)]
struct User {
    id: Option<i64>,
    email: String,
    created_at: OffsetDateTime,
}
```

Joining `"users"` to `"projects"` now is possible and can produce interesting queries, for example:

=== "Rust"
    ```rust
    let beginners = User::all()
      .join::<Project>()
      .filter("project_name", "Rust Programming Language: Introduction")
      .filter("completed", false)
      .fetch_all(&mut conn)
      .await?;
    ```
=== "SQL"
    ```postgresql
    SELECT "users".* FROM "users"
    INNER JOIN "projects" ON "users"."id" = "projects"."user_id"
    WHERE "project_name" = $1 AND "completed_at" = $2
    ```

## Additional relationships

`belongs_to` and `has_many` are the most common relationships, but it's possible to define more. For example, the "has one" relationship where one
row in a table has _only one_ row related to it in another table is a common relationship which doesn't have its own macro annotation.

To implement this relationship, specify the `belongs_to` relationship, and add a `UNIQUE` constraint on the foreign key referring to that table. For example,
if we wanted to allow the users of our fictional web app to have only one project, we can enforce this by altering the `"projects"` table:

```postgresql
ALTER TABLE "projects" ALTER COLUMN "user_id" UNIQUE;
```

This creates a unique index on that column, so if a user attempts to create a second project, the database will return an error.

## Joining multiple tables

Joining across multiple tables is possible as long as there exists at least one relationship between all tables in the query. For example,
if we had another model called `Goal` which belongs to a `Project`, we would be able to join `"users"` to `"goals"` by going through `"projects"` first:

```rust
#[derive(Clone, macros::Model)]
#[belongs_to(Project)]
struct Goal {
    id: Option<i64>,
    project_id: i64,
    priority: i64,
    goal_name: String,
    achived: bool,
}

#[derive(Clone, macros::Model)]
#[has_many(Goal)]
struct Project { /* ... */ }
```

The join will have to use the `join_nested` function instead, since `User` isn't directly related to `Goal`:

=== "Rust"
    ```rust
    let users = User::all()
        .join_nested(Project::join::<Goal>())
        .filter("goal_name", "Learn a lot")
        .fetch_all(&mut conn)
        .await?;
    ```
=== "SQL"
    ```postgresql
    SELECT "users".* FROM "users"
    INNER JOIN "projects" ON "projects"."user_id" = "users"."id"
    INNER JOIN "goals" ON "goals"."project_id" = "projects"."id"
    WHERE "goal_name" = $1
    ```
