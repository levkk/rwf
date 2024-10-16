# Migrations

Migrations are a systematic way of changing your database schema. They allow to add/remove/change database tables in a predictable and ordered
way, and to make sure all versions of your database, in development and in production, have the same schema.

Rwf supports migrations out of the box. To use this feature, install Rwf CLI, if you haven't already:

```
cargo install rwf-cli
```

The CLI should be available globally. You can check if it's working correctly by running:

```
rwf-cli --help
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
migration, run the following:

```
rwf-cli migrate add --name "<migration name>"
```

The migration name is optional, and by default the migration will be "unnamed", but it's nice to name it something recognizable, to help others
working on the project (and the future you) to know what's being changed.

### Writing a migration

The `migrate add` command creates two files in the `migrations` folder: the "up" migration and the "down" migration. The "up" migration contains
the desired changes to the database schema, while the "down" migration contains commands to revert those changes. Having the "down" migration is
very helpful in case the migration doesn't work in production and you need to revert your changes and try again.
