pub mod model;
use crate::model::{get_connection, Model};
pub use model::Migration;

use super::Error;

use std::collections::HashMap;
use std::env::current_dir;
use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;
use tokio::fs::{read_dir, read_to_string};
use tracing::{error, info};

pub struct Migrations {
    migrations: Vec<Migration>,
}

static RE: Lazy<Regex> =
    Lazy::new(|| Regex::new("([0-9]+)_([a-zA-Z0-9_]+).(up|down).sql").expect("migration regex"));

#[derive(Debug, PartialEq)]
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
        self.up.len() == 1 && self.down.len() == 1
    }

    fn missing(&self) -> &str {
        if self.up.len() != 1 {
            "up"
        } else {
            "down"
        }
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
            return Err(Error::MigrationError(format!(
                r#""{}" is not a valid migration file name"#,
                name
            )));
        }
        let captures = RE.captures(name).unwrap();
        println!("{:?}", captures);
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

    pub async fn sync(direction: Direction) -> Result<Self, Error> {
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

        // let mut migrations = vec![];

        for (name, check) in checks {
            if !check.valid() {
                error!(
                    r#"migration "{}" is missing the {} file"#,
                    name,
                    check.missing()
                );
                return Err(Error::MigrationError("migrations file missing".into()));
            } else {
                // match direction {
                //     Direction::Up => migrations.push(check.up.pop().unwrap()),
                //     Direction::Down => migrations.push(check.down.pop().unwrap()),
                // }
            }
        }

        todo!()

        // Ok(Self {
        //     migrations,
        // })
    }

    async fn apply(mut self, direction: Direction) -> Result<Self, Error> {
        let migrations = match direction {
            Direction::Up => self.migrations.into_iter().collect::<Vec<_>>(),
            Direction::Down => self.migrations.into_iter().rev().collect::<Vec<_>>(),
        };

        for migration in &migrations {
            if migration.applied_at.is_some() {
                info!(r#"migration "{}" already applied"#, migration.name);
                continue;
            }

            let path = Self::root_path()?.join(format!(
                "{}.{}.sql",
                migration.name,
                match direction {
                    Direction::Up => "up",
                    Direction::Down => "down",
                }
            ));
        }

        Self::load().await
    }
}

pub async fn migrate() -> Result<Vec<Migration>, super::Error> {
    Migration::sync().await
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
