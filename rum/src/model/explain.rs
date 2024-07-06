use super::{FromRow, Model};

#[derive(Debug, Clone)]
pub struct Explain {
    plan: String,
}

impl Model for Explain {
    fn table_name() -> String {
        "pg_catalog".into()
    }
}

impl std::fmt::Display for Explain {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.plan)
    }
}

impl FromRow for Explain {
    fn from_row(row: &tokio_postgres::row::Row) -> Self {
        let plan = row.get(0);
        Self { plan }
    }
}
