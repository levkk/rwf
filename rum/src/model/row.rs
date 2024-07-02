use super::{Error, ToValue, Value};
use tokio_postgres::types::FromSql;

use std::collections::HashMap;

#[derive(Debug)]
pub struct Row {
    row: tokio_postgres::Row,
    changed: HashMap<String, Value>,
}

impl Row {
    pub fn new(row: tokio_postgres::Row) -> Self {
        Self {
            row,
            changed: HashMap::new(),
        }
    }

    pub fn get<'a, T>(&'a self, name: &str) -> Result<T, Error>
    where
        T: FromSql<'a>,
    {
        Ok(self.row.try_get(name)?)
    }

    pub fn set(&mut self, name: &str, value: impl ToValue) {
        self.changed.insert(name.to_string(), value.to_value());
    }
}
