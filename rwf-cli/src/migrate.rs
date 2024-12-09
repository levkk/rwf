use rwf::model::migrations::{Direction, Migrations};
use std::path::Path;
use time::OffsetDateTime;

use regex::Regex;
use tokio::fs::{create_dir, File};

use crate::{logging::created, util::package_info};

pub async fn migrate(version: Option<i64>) {
    let info = package_info().await.expect("couldn't get package info");

    if info.rwf_auth {
        rwf_auth::migrate()
            .await
            .expect("rwf-auth migrations failed to apply");
    }

    let migrations = Migrations::sync(None)
        .await
        .expect("failed to sync migrations");

    migrations
        .apply(Direction::Up, version)
        .await
        .expect("failed to apply migrations");
}

pub async fn revert(version: Option<i64>) {
    let migrations = Migrations::sync(None)
        .await
        .expect("failed to sync migrations");
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
        created(format!("created \"migrations\" directory"));
    }

    for suffix in ["up", "down"] {
        let name = path.join(format!("{}_{}.{}.sql", version, name, suffix));
        File::create(&name)
            .await
            .expect("failed to create migration file");
        created(format!("\"{}\"", name.display()));
    }
}
