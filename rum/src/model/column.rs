use super::{Escape, ToSql};

#[derive(Debug, Clone)]
pub struct Column {
    table_name: String,
    column_name: String,
}

impl ToSql for Column {
    fn to_sql(&self) -> String {
        if self.table_name.is_empty() {
            format!(r#""{}""#, self.column_name.escape())
        } else {
            format!(
                r#""{}"."{}""#,
                self.table_name.escape(),
                self.column_name.escape()
            )
        }
    }
}

impl Column {
    pub fn new(table_name: impl ToString, column_name: impl ToString) -> Self {
        Self {
            table_name: table_name.to_string(),
            column_name: column_name.to_string(),
        }
    }

    pub fn name(column_name: impl ToString) -> Self {
        Self::new("", column_name)
    }
}

#[derive(Debug, Default)]
pub struct Columns {
    columns: Vec<Column>,
}

impl ToSql for Columns {
    fn to_sql(&self) -> String {
        if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns
                .iter()
                .map(|column| column.to_sql())
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}
