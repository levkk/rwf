use crate::macros;
use crate::model::pool::Transaction;
use crate::model::{AssociationType, Error, Escape, Insert, Model, Query, ToSql, ToValue, Value};
use crate::prelude::ToConnectionRequest;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tokio_postgres::Row;
use tracing::info;

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
    pub(super) id: i64,
    pub(super) name: String,
    pub(super) up: String,
    pub(super) down: String,
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
            name: row.try_get("name")?,
            up: row.try_get("up")?,
            down: row.try_get("down")?,
        })
    }
}

impl Model for RwfDatabaseSchema {
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum SchemaState {
    UNKNOWN,
    CREATED,
    REMOVED,
    APPLIED,
    UNAPPLIED,
}

impl std::fmt::Display for SchemaState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::UNKNOWN => write!(f, "UNKNOWN"),
            Self::CREATED => write!(f, "CREATED"),
            Self::REMOVED => write!(f, "REMOVED"),
            Self::APPLIED => write!(f, "APPLIED"),
            Self::UNAPPLIED => write!(f, "UNAPPLIED"),
        }
    }
}

impl std::str::FromStr for SchemaState {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "UNKNOWN" => Ok(Self::UNKNOWN),
            "CREATED" => Ok(Self::CREATED),
            "REMOVED" => Ok(Self::REMOVED),
            "APPLIED" => Ok(Self::APPLIED),
            "UNAPPLIED" => Ok(Self::UNAPPLIED),
            _ => Err("Unknown SchemaState"),
        }
    }
}
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
    fn removed(&self) -> bool {
        self.eq(&Self::REMOVED)
    }
}
impl ToValue for SchemaState {
    fn to_value(&self) -> Value {
        Value::String(self.to_string())
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
                (id bigint primary key,
                name varchar(255) not null unique,
                up text not null,
                down text not null)"#,
            Self::table_name()
        )
    }
    pub(crate) async fn up_stmts(
        &self,
        tx: &mut Transaction,
        log_queries: bool,
    ) -> Result<(), Error> {
        let stmts = self
            .up
            .split(";")
            .map(|s| s.trim())
            .filter(|stmt| !stmt.is_empty())
            .collect::<Vec<_>>();
        let migrate = RwfSchemaMigration::create(&[
            (Self::foreign_key(), self.id()),
            ("state", SchemaState::APPLIED.to_value()),
        ]);
        for stmt in stmts {
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
        let stmts = self
            .down
            .split(";")
            .map(|s| s.trim())
            .filter(|stmt| !stmt.is_empty())
            .collect::<Vec<_>>();
        let migrate = RwfSchemaMigration::create(&[
            (Self::foreign_key(), self.id()),
            ("state", SchemaState::UNAPPLIED.to_value()),
        ]);
        for stmt in stmts {
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
        let mut values = vec![self.id()];
        values.extend(self.values());
        let applied: Query<Self> = Query::Insert(Insert::from_columns(
            Self::all_columns().as_slice(),
            values.as_slice(),
        ));
        let migrate = RwfSchemaMigration::create(&[
            (Self::foreign_key(), self.id()),
            ("state", SchemaState::CREATED.to_value()),
        ]);
        if log_queries {
            info!("{}", applied.to_sql());
        }
        applied.fetch(&mut (*tx)).await?;
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
    pub(crate) async fn max_active_version(
        conn: impl ToConnectionRequest<'_>,
    ) -> Result<Option<Self>, Error> {
        Self::find_by_sql("SELECT * FROM rwf_database_schema INNER JOIN (SELECT migrations.rwf_database_schema_id from (select max(id) as id , rwf_database_schema_id from rwf_schema_migration group by rwf_database_schema_id) as migrations inner join rwf_schema_migration ON migrations.id = rwf_schema_migration.id where state = 'APPLIED') as mapplied ON rwf_database_schema.id = rwf_database_schema_id ORDER By id DESC LIMIT 1;", &[]).fetch_optional(conn).await
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
