//! Implements database migrations, a deterministic mechanism to change the database schema.
pub mod bootstrap;
mod migrations;
pub mod model;

use crate::config::get_config;
use crate::model::{get_connection, get_pool, start_transaction, Model, ToValue};
use model::Migration;

use super::Error;

use std::collections::HashMap;
use std::env::current_dir;
use std::path::{Path, PathBuf};

use crate::model::migrations::bootstrap::RwfDatabaseSchema;
use crate::model::pool::Transaction;
use once_cell::sync::Lazy;
use regex::Regex;
use time::OffsetDateTime;
use tokio::fs::{read_dir, read_to_string};
use tracing::{error, info};

/// Migrations found in the `"migrations"` folder. Some of them
/// may not be applied yet.
pub struct Migrations {
    migrations: Vec<Migration>,
}

static RE: Lazy<Regex> =
    Lazy::new(|| Regex::new("([0-9]+)_([a-zA-Z0-9_]+).(up|down).sql").expect("migration regex"));

/// Migration direction: up means to apply the migration, down means to revert it.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Default)]
struct Check {
    up: Vec<MigrationFile>,
    down: Vec<MigrationFile>,
}

impl Check {
    fn add(&mut self, file: MigrationFile) {
        match file.direction {
            Direction::Up => self.up.push(file),
            Direction::Down => self.down.push(file),
        }
    }

    fn valid(&self) -> bool {
        self.up.len() == 1
            && self.down.len() == 1
            && self.up.first().unwrap().version == self.down.first().unwrap().version
    }

    fn missing(&self) -> &str {
        if self.up.len() != 1 {
            "up"
        } else {
            "down"
        }
    }

    fn version(&self) -> u64 {
        self.up.first().unwrap().version
    }
}

#[derive(Debug)]
struct MigrationFile {
    version: u64,
    name: String,
    direction: Direction,
}

impl MigrationFile {
    fn parse(name: &str) -> Result<Self, Error> {
        if !RE.is_match(name) {
            error!(r#""{}" is not a valid migration file name"#, name);
            return Err(Error::MigrationError(format!(
                r#""{}" is not a valid migration file name"#,
                name
            )));
        }
        let captures = RE.captures(name).unwrap();
        let version = captures.get(1).unwrap().as_str().parse().unwrap();
        let name = captures.get(2).unwrap();
        let direction = captures.get(3).unwrap();

        Ok(Self {
            version,
            name: name.as_str().to_owned(),
            direction: match direction.as_str() {
                "up" => Direction::Up,
                "down" => Direction::Down,
                d => panic!("unknown direction: {}", d),
            },
        })
    }
}

impl Migrations {
    fn root_path() -> Result<PathBuf, Error> {
        let path = current_dir()?.join(Path::new("migrations"));

        if !path.is_dir() {
            info!(r#"No migrations available, skipping"#);
            Err(Error::MigrationError(
                "migrations folder does not exist".into(),
            ))
        } else {
            Ok(path)
        }
    }

    async fn load() -> Result<Self, Error> {
        let mut conn = get_connection().await?;
        let migrations = Migration::all().fetch_all(&mut conn).await?;

        Ok(Self { migrations })
    }

    /// Read the `"migrations"` folder and sync all migrations
    /// to the `"rwf_migrations"` table in the database. This does not
    /// actually apply the migrations, only makes sure the entries in the folder
    /// match the database table.
    pub async fn sync() -> Result<Self, Error> {
        let checks = if let Ok(root_path) = Self::root_path() {
            let mut checks = HashMap::new();

            let mut dir_entries = read_dir(root_path).await?;
            while let Some(dir_entry) = dir_entries.next_entry().await? {
                let metadata = dir_entry.metadata().await?;
                if metadata.is_file() {
                    let file_name = dir_entry
                        .file_name()
                        .to_str()
                        .expect("migration OsString")
                        .to_string();

                    // Skip hidden files
                    if file_name.starts_with(".") {
                        continue;
                    }

                    let file = MigrationFile::parse(&file_name)?;
                    let entry = checks
                        .entry(file.name.clone())
                        .or_insert_with(Check::default);
                    entry.add(file);
                }
            }

            checks
        } else {
            HashMap::new()
        };

        let mut conn = start_transaction().await?;

        let mut migrations = vec![];

        for (name, check) in checks {
            if !check.valid() {
                error!(
                    r#"migration "{}" is missing the {} file"#,
                    name,
                    check.missing()
                );
                return Err(Error::MigrationError("migrations file missing".into()));
            } else {
                let migration = Migration::filter("name", name)
                    .filter("version", check.version() as i64)
                    .find_or_create()
                    .fetch(&mut conn)
                    .await?;
                migrations.push(migration);
            }
        }

        migrations.sort_by_key(|migration| migration.version);

        conn.commit().await?;

        Ok(Self { migrations })
    }

    /// Apply the migrations, making changes to the database schema.
    ///
    /// The direction argument controllers if we are applying or reverting the migrations. The version
    /// argument means to perform this action up to and including that version.
    pub async fn apply(self, direction: Direction, version: Option<i64>) -> Result<Self, Error> {
        let migrations = match direction {
            Direction::Up => self.migrations.into_iter().collect::<Vec<_>>(),
            Direction::Down => self.migrations.into_iter().rev().collect::<Vec<_>>(),
        };

        let mut stop = false;

        for mut migration in migrations {
            if stop {
                break;
            }

            stop = Some(migration.version) == version;

            let (skip, message) = match direction {
                Direction::Up => (migration.applied_at.is_some(), "applied"),
                Direction::Down => (migration.applied_at.is_none(), "reverted"),
            };

            if skip {
                info!(r#"migration "{}" already {}"#, migration.name(), message);
                continue;
            }

            info!(
                r#"{} migration "{}""#,
                match direction {
                    Direction::Up => "applying",
                    Direction::Down => "reverting",
                },
                migration.name()
            );

            let path = Self::root_path()?.join(migration.path(direction));

            let sql = read_to_string(path).await?;
            let queries = sql
                .split(";")
                .filter(|q| !q.trim().is_empty())
                .map(|q| q.trim().to_string())
                .collect::<Vec<_>>();

            let pool = get_pool();
            let log_queries = get_config().general.log_queries;

            // Execute the migration in a transaction.
            pool.with_transaction(|mut transaction| async move {
                transaction
                    .query_cached("SET LOCAL client_min_messages TO WARNING", &[])
                    .await?;

                for query in queries {
                    if let Err(err) = transaction.client().query(&query, &[]).await {
                        error!(r#"migration "{}" failed: {:?}"#, migration.name(), err);
                        return Err(Error::MigrationError("migration failed".into()));
                    }

                    if log_queries {
                        info!("{}", query);
                    }
                }
                match direction {
                    Direction::Up => migration.applied_at = Some(OffsetDateTime::now_utc()),
                    Direction::Down => migration.applied_at = None,
                };

                let migration = migration.save().fetch(&mut transaction).await?;

                transaction.commit().await?;

                info!(
                    "migration \"{}\" {}",
                    migration.name(),
                    match direction {
                        Direction::Up => "applied",
                        Direction::Down => "reverted",
                    }
                );

                Ok(())
            })
            .await?;
        }

        Self::load().await
    }

    /// Get a list of all migrations currently found in the `"migrations"` folder.
    pub fn migrations(&self) -> &[Migration] {
        &self.migrations
    }

    /// Execute all migrations in the up direction.
    pub async fn migrate() -> Result<Migrations, Error> {
        migrate_internal(None).await?;
        Migrations::sync().await?.apply(Direction::Up, None).await
    }

    /// Execute all migrations in the down direction. **This will effectively
    /// destroy all tables and data in your database.**
    pub async fn flush() -> Result<Migrations, Error> {
        migrate_internal(None).await?;
        Migrations::sync().await?.apply(Direction::Down, None).await
    }
}

/// Ensure all required internal Tables exists
async fn create_schema_tables(tx: &mut Transaction, log_queries: bool) -> Result<(), Error> {
    let query = RwfDatabaseSchema::create_table();
    if log_queries {
        info!("{}", query);
    }
    tx.query_cached(query.as_str(), &[]).await?;

    for query in bootstrap::RwfSchemaMigration::create_table() {
        if log_queries {
            info!("{}", query);
        }
        tx.query_cached(query.as_str(), &[]).await?;
    }
    Ok(())
}
/// Update known Schemas

async fn update_database_schema(tx: &mut Transaction, log_queries: bool) -> Result<(), Error> {
    let migrations = migrations::migrations();
    let new_migrations = match RwfDatabaseSchema::latest_version(&mut (*tx)).await {
        Ok(schema) => migrations
            .into_iter()
            .filter(|migration| migration.id > schema.id)
            .collect::<Vec<_>>(),
        Err(Error::RecordNotFound) => migrations,
        Err(e) => return Err(e),
    };
    for migration in new_migrations {
        migration.create(tx, log_queries).await?;
    }
    Ok(())
}

/// Executes all internal Migrations in down Direction
pub async fn rollback_internal(migration_version: Option<uuid::Uuid>) -> Result<(), Error> {
    let mut tx = start_transaction().await?;
    let log_queries = get_config().general.log_queries;
    create_schema_tables(&mut tx, log_queries).await?;
    update_database_schema(&mut tx, log_queries).await?;
    let active_version = RwfDatabaseSchema::max_active_version(&mut tx).await?;
    let target_version = if let Some(target) = migration_version {
        RwfDatabaseSchema::find_by("migration", target)
            .fetch(&mut tx)
            .await?
            .id()
    } else {
        0.to_value()
    };
    let migrations = if let Some(version) = active_version {
        RwfDatabaseSchema::internal_migrations().filter_lte("id", version.id())
    } else {
        RwfDatabaseSchema::internal_migrations()
    }
    .filter_gte("id", target_version)
    .order(("id", "desc"))
    .fetch_all(&mut tx)
    .await?;

    for migration in migrations {
        migration.down_stmts(&mut tx, log_queries).await?;
    }
    tx.commit().await
}

/// Execute internal migrations in up Direction
pub async fn migrate_internal(migration_version: Option<uuid::Uuid>) -> Result<(), Error> {
    let mut tx = start_transaction().await?;
    let log_queries = get_config().general.log_queries;
    create_schema_tables(&mut tx, log_queries).await?;
    update_database_schema(&mut tx, log_queries).await?;
    let target_version = if let Some(target) = migration_version {
        RwfDatabaseSchema::find_by("migration", target)
            .fetch(&mut tx)
            .await?
            .id()
    } else {
        i64::MAX.to_value()
    };
    let active_version = RwfDatabaseSchema::max_active_version(&mut tx).await?;
    let migrations = if let Some(version) = active_version {
        RwfDatabaseSchema::internal_migrations().filter_gte("id", version.id())
    } else {
        RwfDatabaseSchema::internal_migrations()
    }
    .filter_lte("id", target_version)
    .order(("id", "asc"))
    .fetch_all(&mut tx)
    .await?;

    for migration in migrations {
        migration.up_stmts(&mut tx, log_queries).await?;
    }
    tx.commit().await
}

pub fn info(version: Option<uuid::Uuid>) {
    if version.is_none() {
        for mig in migrations::migrations() {
            eprintln!("{}", mig.description());
        }
    } else {
        let version = version.unwrap();
        for mig in migrations::migrations() {
            if mig.migration == version {
                eprintln!("{}", serde_norway::to_string(&mig).unwrap());
            }
        }
    }
}

/// Execute all migrations in the up direction.
pub async fn migrate() -> Result<Migrations, Error> {
    Migrations::migrate().await
    //Migrations::sync().await?.apply(Direction::Up, None).await
}

/// Execute all migrations in the down direction. **This will effectively
/// destroy all tables and data in your database.**
pub async fn rollback() -> Result<Migrations, Error> {
    Migrations::flush().await
    //Migrations::sync().await?.apply(Direction::Down, None).await
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_migration_file_names() {
        let file = MigrationFile::parse("1234_name_long.up.sql").expect("migration file");
        assert_eq!(file.direction, Direction::Up);
        assert_eq!(file.name.as_str(), "name_long");
        assert_eq!(file.version, 1234);

        let file =
            MigrationFile::parse("1234534_Name_short_long234Adf.down.sql").expect("migration file");
        assert_eq!(file.direction, Direction::Down);
        assert_eq!(file.name.as_str(), "Name_short_long234Adf");
        assert_eq!(file.version, 1234534);
    }
}
