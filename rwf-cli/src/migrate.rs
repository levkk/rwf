use rwf::model::migrations::{Direction, Migrations};
use std::path::Path;
use time::OffsetDateTime;

use regex::Regex;
use tokio::fs::{create_dir, File};

use crate::logging::created;

pub async fn infos(migration: Option<uuid::Uuid>) {
    rwf::model::migrations::info(migration);
}
pub async fn upgrade(version: Option<uuid::Uuid>) {
    rwf::model::migrations::migrate_internal(version)
        .await
        .expect("Failed to apply internal schema migrations.");
}

pub async fn downgrade(version: Option<uuid::Uuid>) {
    rwf::model::migrations::rollback_internal(version)
        .await
        .expect("Failed to rollback internal schema")
}

pub async fn migrate(version: Option<i64>) {
    rwf::model::migrations::migrate_internal(None)
        .await
        .expect("Failed to apply internal migrations.");
    let migrations = Migrations::sync().await.expect("failed to sync migrations");

    migrations
        .apply(Direction::Up, version)
        .await
        .expect("failed to apply migrations");
}

pub async fn revert(version: Option<i64>) {
    let migrations = Migrations::sync().await.expect("failed to sync migrations");
    let version = if let Some(version) = version {
        Some(version)
    } else {
        migrations.migrations().last().map(|v| v.version)
    };

    migrations
        .apply(Direction::Down, version)
        .await
        .expect("failed to apply migrations");
}

pub async fn add(name: &str) {
    let regex = Regex::new("[^a-zA-Z0-9_]").unwrap();
    let name = regex.replace_all(name, "_");
    let version = OffsetDateTime::now_utc().unix_timestamp_nanos();
    let path = Path::new("migrations");

    if !path.exists() {
        create_dir(&path)
            .await
            .expect("cannot create migrations directory");
        created("created \"migrations\" directory".to_string());
    }

    for suffix in ["up", "down"] {
        let name = path.join(format!("{}_{}.{}.sql", version, name, suffix));
        File::create(&name)
            .await
            .expect("failed to create migration file");
        created(format!("\"{}\"", name.display()));
    }
}
