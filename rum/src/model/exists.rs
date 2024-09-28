use super::{FromRow, Model, Value};

#[derive(Debug, Clone)]
pub struct Exists {
    pub count: i64,
}

impl FromRow for Exists {
    fn from_row(row: tokio_postgres::Row) -> Self {
        Self {
            count: row.get("count"),
        }
    }
}

impl Model for Exists {
    fn table_name() -> &'static str {
        unimplemented!()
    }

    fn foreign_key() -> String {
        unimplemented!()
    }

    fn column_names() -> &'static [&'static str] {
        &[]
    }

    fn values(&self) -> Vec<Value> {
        vec![]
    }

    fn id(&self) -> Value {
        Value::Null
    }
}
