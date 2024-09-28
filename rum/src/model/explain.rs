use super::{FromRow, Model, Value};

#[derive(Debug, Clone)]
pub struct Explain {
    plan: String,
}

impl Model for Explain {
    fn table_name() -> &'static str {
        unimplemented!()
    }

    fn foreign_key() -> &'static str {
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

impl std::fmt::Display for Explain {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.plan)
    }
}

impl FromRow for Explain {
    fn from_row(row: tokio_postgres::Row) -> Self {
        let plan = row.get(0);
        Self { plan }
    }
}
