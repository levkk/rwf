//! Represents a single database row for raw queries.
use super::{Error, FromRow, Model, Value};

use std::{collections::HashMap, sync::Arc};

fn de_row<'de, D>(d: D) -> Result<Arc<tokio_postgres::Row>, D::Error> 
where D: serde::Deserializer<'de>
{
    panic!("Desrialize is only implemented as a Marker");
}

#[derive(Debug, Clone, crate::prelude::Deserialize)]
pub struct Row {
    #[serde(deserialize_with="de_row")]
    row: Arc<tokio_postgres::Row>,
}

impl Model for Row {
    fn table_name() -> &'static str {
        "_rwf_rows"
    }

    fn primary_key() -> &'static str {
        "id"
    }

    fn foreign_key() -> &'static str {
        "_rwf_row_id"
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

impl FromRow for Row {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, Error> {
        Ok(Self::new(row))
    }
}

impl std::ops::Deref for Row {
    type Target = tokio_postgres::Row;

    fn deref(&self) -> &Self::Target {
        &self.row
    }
}

impl Row {
    pub fn new(row: tokio_postgres::Row) -> Self {
        Self { row: Arc::new(row) }
    }

    pub fn values(self) -> Result<HashMap<String, Value>, Error> {
        let mut result = HashMap::new();
        for column in self.columns() {
            let name = column.name();
            result.insert(name.to_string(), self.try_get(name)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::super::{Query, ToSql};
    use super::*;

    #[test]
    fn test_random_query() {
        let query = Row::find_by_sql("SELECT 1", &[]);
        assert_eq!(query.to_sql(), "SELECT 1");

        let query = Query::<Row>::select("users");
        assert_eq!(query.to_sql(), "SELECT * FROM \"users\"");
    }
}
