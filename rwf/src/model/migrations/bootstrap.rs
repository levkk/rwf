use crate::macros;
use crate::model::pool::Transaction;
use crate::model::{AssociationType, Error, Escape, Join, Model, Scope, ToSql, ToValue, Value};
use crate::prelude::ToConnectionRequest;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio_postgres::Row;
use tracing::info;

macro_rules! migration_enum {
    ($name:ident, $($opt:ident),*) => {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
        pub(crate) enum $name {
            $($opt),*
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $(
                        $name::$opt => write!(f, stringify!($opt))
                    ),*
                }
            }
        }
        impl std::str::FromStr for $name {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_uppercase().as_str() {
                    $(
                        stringify!($opt) => Ok($name::$opt)
                    ),*
                    , _ => Err(format!("Invalid variant {} for enum {}", s, stringify!($name)))
                }
            }
        }

        impl ToValue for $name {
            fn to_value(&self) -> Value {
                Value::String(self.to_string())
            }
        }
    };
}

migration_enum!(SchemaState, UNKNOWN, CREATED, APPLIED, UNAPPLIED, REMOVED);
migration_enum!(SchemaKind, INTERNAL, FEATURE);

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
pub(crate) struct RwfDatabaseSchema {
    pub(super) id: Option<i64>,
    pub(super) migration: uuid::Uuid,
    pub(super) name: String,
    pub(super) requires: Option<uuid::Uuid>,
    pub(super) kind: String,
    pub(super) up: Vec<String>,
    pub(super) down: Vec<String>,
    pub(super) description: String,
}

pub(crate) fn parse_database_schema(data: &str) -> RwfDatabaseSchema {
    let yaml: serde_norway::Value = serde_norway::from_str(data.trim()).unwrap();
    RwfDatabaseSchema {
        id: None,
        migration: yaml
            .get("migration")
            .unwrap()
            .as_str()
            .map(|uuid| uuid::Uuid::parse_str(uuid).unwrap())
            .unwrap(),
        name: yaml.get("name").unwrap().as_str().unwrap().to_string(),
        requires: yaml
            .get("requires")
            .map(|value| uuid::Uuid::parse_str(value.as_str().unwrap()).unwrap()),
        kind: yaml.get("kind").unwrap().as_str().unwrap().to_owned(),
        up: yaml
            .get("up")
            .unwrap()
            .as_sequence()
            .unwrap()
            .into_iter()
            .map(|val| val.as_str().unwrap().to_string())
            .collect(),
        down: yaml
            .get("down")
            .unwrap()
            .as_sequence()
            .unwrap()
            .into_iter()
            .map(|val| val.as_str().unwrap().to_string())
            .collect(),
        description: yaml
            .get("description")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string(),
    }
}

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
pub(crate) struct RwfSchemaMigration {
    pub(super) id: Option<i64>,
    pub(super) rwf_database_schema_id: i64,
    pub(super) state: String,
    pub(super) ts: OffsetDateTime,
}

impl crate::model::FromRow for RwfDatabaseSchema {
    fn from_row(row: Row) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.try_get("id")?,
            migration: row.try_get("migration")?,
            name: row.try_get("name")?,
            requires: row.try_get("requires")?,
            kind: row.try_get("kind")?,
            up: row.try_get("up")?,
            down: row.try_get("down")?,
            description: row.try_get("description")?,
        })
    }
}

impl Model for RwfDatabaseSchema {
    fn table_name() -> &'static str {
        "rwf_database_schema"
    }

    fn column_names() -> &'static [&'static str] {
        &[
            "migration",
            "name",
            "requires",
            "kind",
            "up",
            "down",
            "description",
        ]
    }

    fn id(&self) -> Value {
        self.id.to_value()
    }

    fn values(&self) -> Vec<Value> {
        vec![
            self.migration.to_value(),
            self.name.to_value(),
            self.requires.to_value(),
            self.kind.to_value(),
            self.up
                .iter()
                .map(|stmt| stmt.to_value())
                .collect::<Vec<_>>()
                .as_slice()
                .to_value(),
            self.down
                .iter()
                .map(|stmt| stmt.to_value())
                .collect::<Vec<_>>()
                .as_slice()
                .to_value(),
            self.description.to_value(),
        ]
    }

    fn foreign_key() -> &'static str {
        "rwf_database_schema_id"
    }
}

impl crate::model::FromRow for RwfSchemaMigration {
    fn from_row(row: Row) -> Result<Self, Error>
    where
        Self: Sized,
    {
        Ok(Self {
            id: row.try_get("id")?,
            rwf_database_schema_id: row.try_get("rwf_database_schema_id")?,
            state: row.try_get("state")?,
            ts: row.try_get("ts")?,
        })
    }
}
impl Model for RwfSchemaMigration {
    fn table_name() -> &'static str {
        "rwf_schema_migration"
    }
    fn column_names() -> &'static [&'static str] {
        &["rwf_database_schema_id", "action", "ts"]
    }

    fn id(&self) -> Value {
        self.id.to_value()
    }

    fn values(&self) -> Vec<Value> {
        vec![
            self.rwf_database_schema_id.to_value(),
            self.state.to_value(),
            self.ts.to_value(),
        ]
    }

    fn foreign_key() -> &'static str {
        "rwf_schema_migration_id"
    }
}

impl crate::model::Association<RwfSchemaMigration> for RwfDatabaseSchema {
    fn association_type() -> AssociationType {
        AssociationType::HasMany
    }
}

impl crate::model::Association<RwfDatabaseSchema> for RwfSchemaMigration {}

#[allow(unused)]
impl SchemaState {
    fn active(&self) -> bool {
        self.eq(&Self::APPLIED)
    }
    fn deactivated(&self) -> bool {
        self.eq(&Self::UNAPPLIED)
    }
    fn created(&self) -> bool {
        self.eq(&Self::CREATED)
    }
}

impl From<&RwfSchemaMigration> for SchemaState {
    fn from(value: &RwfSchemaMigration) -> Self {
        std::str::FromStr::from_str(value.state.as_str()).unwrap()
    }
}

#[allow(unused)]
impl RwfDatabaseSchema {
    pub(crate) fn create_table() -> String {
        format!(
            r#"CREATE TABLE IF NOT EXISTS "{}"
                (id bigserial primary key,
                migration uuid not null unique,
                name varchar(255) not null unique,
                requires uuid default null REFERENCES "{}"(migration) ON UPDATE CASCADE ON DELETE CASCADE,
                kind varchar(15) not null,
                up text[] not null,
                down text[] not null,
                description text not null)"#,
            Self::table_name(),
            Self::table_name()
        )
    }
    pub(crate) fn description(&self) -> String {
        format!(
            "Migration '{}'\t\t--\t\t{}\t\t{}",
            self.migration, self.name, self.description
        )
    }
    pub(crate) async fn up_stmts(
        &self,
        tx: &mut Transaction,
        log_queries: bool,
    ) -> Result<(), Error> {
        info!("Apply InternalMiigration {}\t{}", self.name, self.migration);
        let migrate = RwfSchemaMigration::create(&[
            (Self::foreign_key(), self.id()),
            ("state", SchemaState::APPLIED.to_value()),
        ]);
        for stmt in &self.up {
            if log_queries {
                info!("{}", stmt);
            }
            tx.query_cached(&stmt, &[]).await?;
        }
        migrate.fetch(tx).await?;
        Ok(())
    }
    pub(crate) async fn down_stmts(
        &self,
        tx: &mut Transaction,
        log_queries: bool,
    ) -> Result<(), Error> {
        info!(
            "Revert InternalMiigration {}\t{}",
            self.name, self.migration
        );
        let migrate = RwfSchemaMigration::create(&[
            (Self::foreign_key(), self.id()),
            ("state", SchemaState::UNAPPLIED.to_value()),
        ]);
        for stmt in &self.down {
            if log_queries {
                info!("{}", stmt);
            }
            tx.query_cached(stmt, &[]).await?;
        }
        migrate.fetch(tx).await?;
        Ok(())
    }
    pub(crate) async fn create(
        &self,
        tx: &mut Transaction,
        log_queries: bool,
    ) -> Result<(), Error> {
        let applied = self.clone().save();
        if log_queries {
            info!("{}", applied.to_sql());
        }
        let applied = applied.fetch(&mut (*tx)).await?;
        let migrate = RwfSchemaMigration::create(&[
            (Self::foreign_key(), applied.id()),
            ("state", SchemaState::CREATED.to_value()),
        ]);

        if log_queries {
            info!("{}", migrate.to_sql());
        }
        migrate.fetch(tx).await?;

        Ok(())
    }
    pub(crate) async fn remove(
        &self,
        tx: &mut Transaction,
        log_queries: bool,
    ) -> Result<(), Error> {
        let migrate = RwfSchemaMigration::create(&[
            (Self::foreign_key(), self.id()),
            ("state", SchemaState::REMOVED.to_value()),
        ]);
        let removed = self.destroy();

        if log_queries {
            info!("{}", migrate.to_sql());
        }
        migrate.fetch(&mut (*tx)).await?;
        if log_queries {
            info!("{}", removed.to_sql());
        }
        removed.fetch(&mut (*tx)).await?;
        Ok(())
    }
    pub(crate) async fn state(
        &self,
        conn: impl ToConnectionRequest<'_>,
    ) -> Result<SchemaState, Error> {
        match RwfSchemaMigration::all()
            .join::<Self>()
            .filter(Self::foreign_key(), self.id())
            .order(("id", "desc"))
            .take_one()
            .fetch_optional(conn)
            .await
        {
            Ok(Some(mig)) => Ok(SchemaState::from(&mig)),
            Ok(None) => Ok(SchemaState::UNKNOWN),
            Err(e) => Err(e),
        }
    }
    pub(crate) async fn latest_version(conn: impl ToConnectionRequest<'_>) -> Result<Self, Error> {
        Self::all()
            .order(("id", "desc"))
            .take_one()
            .fetch(conn)
            .await
    }
    pub(crate) fn max_active_version() -> Scope<Self> {
        Self::internal_migrations()
            .with(
                RwfSchemaMigration::all()
                    .select_aggregated(&[("id", "MAX", Some("max"))])
                    .group_by(&["rwf_database_schema_id"]),
                "latest",
            )
            .with(
                RwfSchemaMigration::all()
                    .add_join(Join::new(
                        RwfSchemaMigration::table_name(),
                        "latest",
                        "max",
                        "id",
                    ))
                    .filter("state", SchemaState::APPLIED.to_value())
                    .last_one(),
                "active",
            )
            .add_join(Join::new(
                RwfDatabaseSchema::table_name(),
                "active",
                "rwf_database_schema_id",
                "id",
            ))
    }

    pub(crate) fn internal_migrations() -> Scope<Self> {
        Self::filter("kind", SchemaKind::INTERNAL.to_value())
    }
    pub(crate) fn internal_migration_root() -> Scope<Self> {
        Self::internal_migrations()
            .first_one()
            .filter("requires", Value::Null)
    }
    pub(crate) fn full_migration_chain(target: Option<uuid::Uuid>) -> Scope<Self> {
        if let Some(target) = target {
            Self::internal_migrations()
                .filter("migration", target.to_value())
                .union(Self::all().add_join(Join::new(
                    Self::table_name(),
                    "recurse",
                    "requires",
                    "migration",
                )))
                .select_recursive_with("recurse")
        } else {
            Self::internal_migration_root()
                .union(Self::internal_migrations().add_join(Join::new(
                    Self::table_name(),
                    "recurse",
                    "migration",
                    "requires",
                )))
                .select_recursive_with("recurse")
        }
    }

    pub(crate) fn migration_chain(target: Option<uuid::Uuid>) -> Scope<Self> {
        Self::full_migration_chain(target).except(Self::max_active_version())
    }
}

impl RwfSchemaMigration {
    pub(crate) fn create_table() -> Vec<String> {
        vec![
            format!(
                r#"CREATE TABLE IF NOT EXISTS "{}" (
            id bigserial primary key,
            "{}" BIGINT NOT NULL REFERENCES {}(id) ON UPDATE CASCADE ON DELETE SET NULL,
            state varchar(15) not null,
            ts TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
                Self::table_name().escape(),
                RwfDatabaseSchema::foreign_key().escape(),
                RwfDatabaseSchema::table_name().escape()
            ),
            format!(
                r#"CREATE INDEX IF NOT EXISTS rwf_schema_migration_database_schema ON "{}"("{}")"#,
                Self::table_name().escape(),
                RwfDatabaseSchema::foreign_key().escape()
            ),
            format!(
                r#"CREATE INDEX IF NOT EXISTS rwf_schema_migration_state ON "{}"(state)"#,
                Self::table_name().escape()
            ),
        ]
    }
}

macros::generate_internal_migrations!("rwf/src/model/migrations/", "bootstrap", "migrations.rs");
