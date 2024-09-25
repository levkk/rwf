pub mod model;
use crate::model::{get_connection, get_pool, Model};
pub use model::Migration;

use super::Error;

use std::collections::HashMap;
use std::env::current_dir;
use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;
use time::OffsetDateTime;
use tokio::fs::{read_dir, read_to_string};
use tracing::{error, info};

pub struct Migrations {
    migrations: Vec<Migration>,
}

static RE: Lazy<Regex> =
    Lazy::new(|| Regex::new("([0-9]+)_([a-zA-Z0-9_]+).(up|down).sql").expect("migration regex"));

#[derive(Debug, PartialEq, Copy, Clone)]
enum Direction {
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
        let path = PathBuf::from(current_dir()?.join(Path::new("migrations")));

        if !path.is_dir() {
            error!(
                r#""{}" folder does not exist, did you create the project using rum-cli?"#,
                path.display()
            );
            Err(Error::MigrationError(
                "migrations folder does not exist".into(),
            ))
        } else {
            Ok(path)
        }
    }

    pub async fn load() -> Result<Self, Error> {
        let root_path = Self::root_path()?;
        let conn = get_connection().await?;
        let migrations = Migration::all().fetch_all(&conn).await?;

        Ok(Self { migrations })
    }

    pub async fn sync() -> Result<Self, Error> {
        let root_path = Self::root_path()?;
        let mut checks = HashMap::new();

        let mut dir_entries = read_dir(root_path).await?;
        while let Some(dir_entry) = dir_entries.next_entry().await? {
            let metadata = dir_entry.metadata().await?;
            if metadata.is_file() {
                let file = MigrationFile::parse(
                    dir_entry.file_name().to_str().expect("migration OsString"),
                )?;
                let entry = checks
                    .entry(file.name.clone())
                    .or_insert_with(Check::default);
                entry.add(file);
            }
        }

        let conn = get_connection().await?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS rum_migrations (
            id BIGSERIAL PRIMARY KEY,
            version BIGINT NOT NULL,
            name VARCHAR UNIQUE NOT NULL,
            applied_at TIMESTAMPTZ
        )",
            &[],
        )
        .await?;

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
                    .fetch(&conn)
                    .await?;
                migrations.push(migration);
            }
        }

        migrations.sort_by_key(|migration| migration.version);

        Ok(Self { migrations })
    }

    async fn apply(mut self, direction: Direction) -> Result<Self, Error> {
        let migrations = match direction {
            Direction::Up => self.migrations.into_iter().collect::<Vec<_>>(),
            Direction::Down => self.migrations.into_iter().rev().collect::<Vec<_>>(),
        };

        for mut migration in migrations {
            let (skip, message) = match direction {
                Direction::Up => (migration.applied_at.is_some(), "applied"),
                Direction::Down => (migration.applied_at.is_none(), "reverted"),
            };

            if skip {
                info!(r#"migration "{}" already {}"#, migration.name, message);
                continue;
            }

            info!(
                r#"{} migration "{}""#,
                match direction {
                    Direction::Up => "applying",
                    Direction::Down => "reverting",
                },
                migration.version
            );

            let path = Self::root_path()?.join(migration.path(direction));

            let sql = read_to_string(path).await?;
            let queries = sql
                .split(";")
                .filter(|q| !q.trim().is_empty())
                .map(|q| q.trim().to_string())
                .collect::<Vec<_>>();

            let pool = get_pool();

            // Execute the migration in a transaction.
            pool.with_transaction(|transaction| async move {
                for query in queries {
                    if let Err(err) = transaction.execute(&query, &[]).await {
                        error!(r#"migration "{}" failed: {:?}"#, migration.name, err);
                        return Err(Error::MigrationError("migration failed".into()));
                    }
                }
                match direction {
                    Direction::Up => migration.applied_at = Some(OffsetDateTime::now_utc()),
                    Direction::Down => migration.applied_at = None,
                };

                migration.save().execute(&transaction).await?;

                transaction.commit().await?;

                Ok(())
            })
            .await?;
        }

        Self::load().await
    }
}

pub async fn migrate() -> Result<Migrations, Error> {
    Migrations::sync().await?.apply(Direction::Up).await
}

pub async fn rollback() -> Result<Migrations, Error> {
    Migrations::sync().await?.apply(Direction::Down).await
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
