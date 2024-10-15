# Rwf CLI

Rwf comes with its own command-line interface (CLI) which helps managing projects.

## Installation

Install the CLI using Cargo:

```
$ cargo install rwf-cli
```

If you have configured Cargo correctly, you should be able to use the CLI directly:

```
$ rwf-cli --help
```

If not, add `~/.cargo/bin/` to your `PATH`.

## Commands

Rwf CLI supports the following features:

- migrations
- project setup

### Migrations

#### Adding a migration

```
$ rwf migrate add --name "name_of_your_migration"
```

#### Running migrations

```
$ rwf migrate run
```

#### Reverting the latest migration

```
$ rwf migrate revert
```

#### Re-create database

This will revert all migrations (**deleting all data**) and re-create all tables and indexes:

```
$ rwf migrate flush
```
