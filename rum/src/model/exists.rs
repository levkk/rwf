use super::{FromRow, Model};

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
    fn table_name() -> String {
        unimplemented!()
    }

    fn foreign_key() -> String {
        unimplemented!()
    }
}
