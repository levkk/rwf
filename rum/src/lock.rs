use crate::model::{Column, Error, FromRow, Model, Pool, ToValue, Value};
use time::OffsetDateTime;

#[derive(Clone)]
pub struct Lock {
    id: Option<i64>,
    name: String,
    created_at: OffsetDateTime,
    expires_at: OffsetDateTime,
    database_now: OffsetDateTime,
}

impl FromRow for Lock {
    fn from_row(row: tokio_postgres::Row) -> Result<Lock, Error> {
        Ok(Lock {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            created_at: row.try_get("created_at")?,
            expires_at: row.try_get("expires_at")?,
            database_now: row.try_get("database_now")?,
        })
    }
}

impl Model for Lock {
    fn column_names() -> &'static [&'static str] {
        &["name", "created_at", "expires_at"]
    }

    fn values(&self) -> Vec<Value> {
        vec![
            self.name.to_value(),
            self.created_at.to_value(),
            self.expires_at.to_value(),
        ]
    }

    fn primary_key() -> &'static str {
        "id"
    }

    fn foreign_key() -> &'static str {
        "rum_lock_id"
    }

    fn table_name() -> &'static str {
        "rum_locks"
    }

    fn id(&self) -> Value {
        self.id.to_value()
    }
}

impl Lock {
    pub fn available(&self) -> bool {
        self.expires_at < self.database_now
    }

    pub async fn new(name: &str) -> Result<Lock, Error> {
        Pool::pool()
            .with_transaction(|mut transaction| async move {
                let lock = Lock::filter("name", name)
                    .column(Column::name("database_now").as_value(Value::function("now")))
                    .lock()
                    .find_or_create()
                    .unique_by(&["name"])
                    .fetch(&mut transaction)
                    .await?;

                Ok(lock)
            })
            .await
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
