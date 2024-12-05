# Add users

!!! note
    This guide is a work-in-progress.

Unless you're building simple demo applications or static informational websites, your web app will need a way for your users to sign up and personalize their experience. There are many ways to accomplish this, and your implementation should be specific to your use case. For example, many web apps allow users to sign up using an OAuth2 provider like [Google](https://developers.google.com/identity/protocols/oauth2) or [GitHub](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/creating-an-oauth-app).

In this guide, we'll cover the most popular and simple way to create user accounts: using a username and a password.

## Username and password

Allowing your users to create accounts in your application using a username and password is pretty universal. Implementing this system requires using all 3 components of the MVC framework: creating a database model to store usernames and password hashes, controllers to process signup and login requests, and views to serve signup and login forms.

Rwf supports all three components natively.

### Create the model

To create a model in Rwf, you need to define the schema in the database and define the model in Rust code. The two should match as closely as possible.

#### Create the schema

Starting with the data model, let's create a simple `"users"` table in your database. This table will store usernames, password hashes, and other metadata about our users, like when their accounts were created.

Creating a table with Rwf should be done by writing a [migration](../../models/migrations.md). This makes sure changes to the database schema are documented and deterministic. To create a migration, use the Rwf CLI:

=== "Command"
    ```
    rwf-cli migrate add -n users
    ```
=== "Output"
    ```
    Created "migrations/1733265254409864495_users.up.sql"
    Created "migrations/1733265254409864495_users.down.sql"
    ```

The migration is empty, so let's create the table by adding it to the `users.up.sql` file:

```postgresql
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR NOT NULL UNIQUE,
    password_hash VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

As mentioned above, our table will store usernames, password hashes, and metadata. The `id` column is the primary key of this table, allowing us to identify our users using a unique number.

!!! note
    Rwf models by default expect the presence of the `id` column, and use it as the primary key.
    This is configurable on a per-model basis, and models can be created without a primary key,
    but this will prevent them from being updated or deleted by the ORM.

Once the schema is ready, create the table in the database by applying the migration:

=== "Command"
    ```
    rwf-cli migrate run
    ```
=== "Output"
    ```
    applying migration "1733265254409864495_users"
    migration "1733265254409864495_users" applied
    ```

#### Define the Rust model

With the schema ready to go, we need to create a Rust struct which we'll use in code to reference the model records. The Rust struct should have the same fields as the columns in our table, and their data types should match as well:

```rust
#[derive(Clone, macros::Model)]
pub struct User {
    id: Option<i64>,
    username: String,
    password_hash: String,
    created_at: OffsetDateTime,
}
```
