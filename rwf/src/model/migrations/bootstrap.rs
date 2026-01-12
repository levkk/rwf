use crate::macros;
use crate::model::{Error, Model, Query, ToValue, Value};
use tokio_postgres::Row;

#[derive(
    Debug,
    crate::prelude::Serialize,
    crate::prelude::Deserialize,
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
)]
pub struct RwfDatabaseSchema {
    pub(super) id: i64,
    pub(super) name: String,
    pub(super) up: String,
    pub(super) down: String,
}

impl crate::model::FromRow for RwfDatabaseSchema {
    fn from_row(row: Row) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            up: row.try_get("up")?,
            down: row.try_get("down")?,
        })
    }
}

impl crate::model::Model for RwfDatabaseSchema {
    fn table_name() -> &'static str {
        "rwf_database_schema"
    }

    fn column_names() -> &'static [&'static str] {
        &["name", "up", "down"]
    }

    fn id(&self) -> Value {
        self.id.to_value()
    }

    fn values(&self) -> Vec<Value> {
        vec![
            self.name.to_value(),
            self.up.to_value(),
            self.down.to_value(),
        ]
    }

    fn foreign_key() -> &'static str {
        "rwf_database_schema_id"
    }
}

impl RwfDatabaseSchema {
    pub(crate) fn create_table() -> crate::model::Query<Self> {
        Query::Raw {
            query: format!(
                r#"CREATE TABLE IF NOT EXISTS "{}"
                (id bigint primary key,
                name varchar(255) not null unique,
                up text not null,
                down text not null)"#,
                Self::table_name()
            ),
            placeholders: Default::default(),
        }
    }
}

macros::generate_internal_migrations!(
    "rwf/src/model/migrations/bootstrap",
    "rwf/src/model/migrations/migrations.rs"
);
