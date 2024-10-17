# Migrations

Migrations are a systematic way of changing your database schema. They allow to add/remove/change database tables in a predictable and ordered
way, and to make sure all versions of your database, in development and in production, have the same schema.

Rwf supports migrations out of the box. To use this feature, install Rwf CLI, if you haven't already:

```
cargo install rwf-cli
```

The CLI should be available globally. You can check if it's working correctly by running:

=== "Command"
    ```
    rwf-cli --help
    ```
=== "Output"
    ```
    Rust Web Framework CLI

    Usage: rwf-cli <COMMAND>

    Commands:
      migrate  Manage migrations
      setup    Setup the project for Rwf
      help     Print this message or the help of the given subcommand(s)

    Options:
      -h, --help     Print help
      -V, --version  Print version
    ```

## Run migrations

If it's your first time setting up a Rwf app, you should run migrations before starting the server. To run the migrations, change directory to
the project root (where `Cargo.toml` is located) and run:

```
rwf-cli migrate run
```

This command will automatically read all migration files in the `migrations` folder, and run the necessary ones in the correct order. If a migration
is already applied to your database, Rwf will skip it and run the next one.

## Create new migration

If you're looking to change your database schema, e.g. by adding a new table, you can do so in a reproducible way by making a migration. To create a new
migration, run the following command:

=== "Command"
    ```
    rwf-cli migrate add --name "<migration name>"
    ```
=== "Output"
    ```
    created "migrations/1729119889028371278_unnamed.up.sql"
    created "migrations/1729119889028371278_unnamed.down.sql"
    ```

Migrations are placed inside the `<PROJECT_ROOT>/migrations` folder. If this folder doesn't exist, `rwf-cli` will create one automatically.

The migration name is optional, and by default the migration will be "unnamed", but it's nice to name it something recognizable, to help others
working on the project (and the future you) to know what's being changed.

### Writing migrations

The `migrate add` command creates two files in the `migrations` folder: the "up" migration and the "down" migration. The "up" migration contains
the desired changes to the database schema, while the "down" migration contains commands to revert those changes.

!!! note
    Having the "down" migration is technically optional but is
    very helpful in case the migration doesn't work in production and you need to revert your changes and try again.
    Additionally, without the correct "down" migration, commands like `migrate flush` won't work correctly.

For example, if we want to add a `"users"` table to our database, we can write the following migration:

=== "Up migration"
    ```postgresql
    CREATE TABLE users (
        id BIGSERIAL PRIMARY KEY,
        email VARCHAR UNIQUE NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );
    ```
=== "Down migration"
    ```postgresql
    DROP TABLE users;
    ```

Once finished, applying the migration can be done by running the following command:

=== "Command"
    ```
    rwf-cli migrate run
    ```
=== "Output"
    ```
    migration "1729119889028371278_unnamed" applied
    ```

## Revert migration

If something went wrong, or you'd like to make more changes without creating another migration (in development), you can revert the last migration by running:

=== "Command"
    ```
    rwf-cli migrate revert
    ```
=== "Output"
    ```
    migration "1729119889028371278_unnamed" reverted
    ```

The `revert` command automatically executes the "down" file for the last migration. If you'd like to revert more than one migration, specify the version to which you want to revert to, by passing the `--version <VERSION>` argument.

Re-running the last migration can be done by running `rwf-cli migrate run` command again.

## Flush the database

In local development, it's sometimes useful to delete everything in your database and start again. To do so, you can run the `rwf-cli migrate flush` command. This command will revert all migrations in reverse order, and re-apply them in normal order again.

!!! warning
    Running `rwf-cli migrate flush` will delete all your data. Never run this command in production.
    To protect against accidental misuse, the command will not do anything unless a `--yes` flag is
    passed to it.
