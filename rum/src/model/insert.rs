use super::{Column, Escape, FromRow, Model, Placeholders, ToColumn, ToSql, ToValue};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Insert<T> {
    table_name: String,
    columns: Vec<Column>,
    pub placeholders: Placeholders,
    marker: PhantomData<T>,
}

impl<T: Model> Insert<T> {
    pub fn new(model: T) -> Self {
        let columns = T::column_names()
            .into_iter()
            .map(|column| Column::name(column))
            .collect();
        let values = model.values();
        let mut placeholders = Placeholders::new();
        for value in values {
            placeholders.add(&value);
        }

        Self {
            table_name: T::table_name().to_string(),
            placeholders,
            columns,
            marker: PhantomData,
        }
    }

    pub fn from_columns(columns: &[impl ToColumn], values: &[impl ToValue]) -> Self {
        let mut placeholders = Placeholders::new();
        for value in values {
            let value = value.to_value();
            placeholders.add(&value);
        }

        Insert {
            table_name: T::table_name().to_string(),
            columns: columns.iter().map(|c| c.to_column().unqualify()).collect(),
            placeholders,
            marker: PhantomData,
        }
    }
}

impl<T: FromRow> ToSql for Insert<T> {
    fn to_sql(&self) -> String {
        let columns = self
            .columns
            .iter()
            .map(|c| c.to_sql())
            .collect::<Vec<_>>()
            .join(", ");
        let placeholders = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            r#"INSERT INTO "{}" ({}) VALUES ({}) RETURNING *"#,
            self.table_name.escape(),
            columns,
            placeholders
        )
    }
}
