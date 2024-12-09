use std::path::PathBuf;

use rwf::model::{Error, Migrations};

pub mod controllers;
pub mod models;

/// Run `rwf-auth` migrations.
pub async fn migrate() -> Result<Migrations, Error> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    rwf::model::migrate(Some(path)).await
}

pub fn migrations_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("migrations")
}
