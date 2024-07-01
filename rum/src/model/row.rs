use super::Error;
use tokio_postgres::types::FromSql;

#[derive(Debug)]
pub struct Row {
    row: tokio_postgres::Row,
}

impl Row {
    pub fn new(row: tokio_postgres::Row) -> Self {
        Self { row }
    }

    pub fn column<'a, T>(&'a self, name: &str) -> Result<T, Error>
    where
        T: FromSql<'a>,
    {
        Ok(self.row.try_get(name)?)
    }
}
