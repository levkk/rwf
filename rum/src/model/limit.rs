use super::ToSql;

#[derive(Debug, Default)]
pub struct Limit {
    limit: Option<i64>,
    offset: Option<i64>,
}

impl ToSql for Limit {
    fn to_sql(&self) -> String {
        let mut limit = String::new();
        if let Some(ref rows) = self.limit {
            limit += format!(" LIMIT {}", rows).as_str();
        }

        if let Some(ref offset) = self.offset {
            limit += format!(" OFFSET {}", offset).as_str();
        }

        limit
    }
}

impl Limit {
    pub fn new(n: usize) -> Self {
        Self {
            limit: Some(n as i64),
            offset: None,
        }
    }
}
