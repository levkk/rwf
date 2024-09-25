use crate::model::{get_connection, get_pool, Error, FromRow, Model, ToValue, Value};
use time::OffsetDateTime;
use tokio::fs::{read_dir, read_to_string};
use tracing::{error, info};

use std::path::{Path, PathBuf};

#[derive(Clone)]
#[allow(dead_code)]
pub struct Migration {
    id: Option<i64>,
    pub name: String,
    pub applied_at: Option<OffsetDateTime>,
}

impl FromRow for Migration {
    fn from_row(row: tokio_postgres::Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            applied_at: row.get("applied_at"),
        }
    }
}

impl Model for Migration {
    fn primary_key() -> String {
        "id".to_string()
    }

    fn table_name() -> String {
        "rum_migrations".to_string()
    }

    fn foreign_key() -> String {
        "rum_migration_id".to_string()
    }

    fn id(&self) -> Value {
        self.id.to_value()
    }

    fn values(&self) -> Vec<Value> {
        vec![self.name.to_value(), self.applied_at.to_value()]
    }

    fn column_names() -> Vec<String> {
        vec!["name".to_string(), "applied_at".to_string()]
    }
}

impl Migration {
    fn path() -> Result<PathBuf, std::io::Error> {
        let cwd = std::env::current_dir()?;
        Ok(cwd.join(Path::new("migrations")).to_owned())
    }

    async fn sql(&self, up: bool) -> Result<Vec<String>, std::io::Error> {
        let postfix = if up { "up" } else { "down" };
        let name = self.name.clone() + &format!(".{}.sql", postfix);
        let path = Self::path()?.join(Path::new(&name));
        let sql = read_to_string(path).await?;
        let sql = sql
            .split(";")
            .filter(|sql| !sql.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>();
        Ok(sql)
    }

    pub async fn sync() -> Result<Vec<Self>, Error> {
        let mut models = vec![];
        let path = Self::path()?;

        if !path.exists() {
            error!("migrations folder does not exist, did you create this project with rum-cli?");
            return Ok(vec![]);
        }
        let mut dir = read_dir(&path).await?;
        while let Some(file) = dir.next_entry().await? {
            if file.file_type().await?.is_file() {
                let name = file
                    .file_name()
                    .as_os_str()
                    .to_str()
                    .expect("OsStr not valid UTF-8")
                    .to_string();
                let name = name.replace(".up.sql", "").replace(".down.sql", "");

                let model = {
                    let conn = get_connection().await?;
                    match Self::find_by("name", &name).fetch(&conn).await {
                        Ok(model) => model,
                        Err(_) => {
                            Self {
                                id: None,
                                name,
                                applied_at: None,
                            }
                            .save()
                            .fetch(&conn)
                            .await?
                        }
                    }
                };

                let model = model.apply().await?;

                models.push(model);
            }
        }

        Ok(models)
    }

    pub async fn revert(mut self) -> Result<Self, Error> {
        if self.applied_at.is_none() {
            return Ok(self);
        }

        let sql = match self.sql(false).await {
            Ok(sql) => sql,
            Err(_) => {
                return Err(Error::MigrationError(format!(
                    "migration \"{}\" not found",
                    self.name
                )))
            }
        };

        let pool = get_pool();
        let transaction = pool.begin().await?;

        for query in sql {
            transaction.execute(&query, &[]).await?;
        }

        self.applied_at = None;
        let migration = self.save().fetch(&transaction).await?;

        transaction.commit().await?;

        Ok(migration)
    }

    pub async fn apply(mut self) -> Result<Self, Error> {
        if self.applied_at.is_some() {
            info!("migration \"{}\" already applied", self.name);
            return Ok(self);
        }

        let sql = match self.sql(true).await {
            Ok(sql) => sql,
            Err(_) => {
                return Err(Error::MigrationError(format!(
                    "migration \"{}\" not found",
                    self.name
                )))
            }
        };

        info!("running migration \"{}\"", self.name);

        let pool = get_pool();
        let transaction = pool.begin().await?;

        for query in sql {
            transaction.execute(&query, &[]).await?;
        }

        self.applied_at = Some(OffsetDateTime::now_utc());
        let migration = self.save().fetch(&transaction).await?;

        transaction.commit().await?;

        Ok(migration)
    }
}
