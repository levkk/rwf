pub mod model;
pub use model::Migration;

use super::Error;

use std::env::current_dir;
use std::path::{Path, PathBuf};
use tracing::{error, info};

pub struct Migrations {
    migrations: Vec<Migration>,
}

impl Migrations {
    fn root_path() -> Result<PathBuf, Error> {
        Ok(PathBuf::from(current_dir()?.join(Path::new("migrations"))))
    }

    pub async fn load() -> Result<Self, Error> {
        let root_path = Self::root_path()?;
        if !root_path.exists() {
            error!(r#""#);
        }
        todo!()
    }
}

pub async fn migrate() -> Result<Vec<Migration>, super::Error> {
    Migration::sync().await
}
