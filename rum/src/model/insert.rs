use super::{Column, Escape, FromRow, Model, Placeholders, ToSql, ToValue};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Insert<T> {
    table_name: String,
    primary_key: String,
    columns: Vec<String>,
    pub placeholders: Placeholders,
    marker: PhantomData<T>,
}

impl<T: Model> Insert<T> {
    pub fn new(model: T) -> Self {
        let columns = T::column_names()
            .into_iter()
            .map(|column| column.escape())
            .collect();
        let values = model.values();
        let mut placeholders = Placeholders::new();
        for value in values {
            placeholders.add(&value);
        }

        Self {
            table_name: T::table_name(),
            primary_key: T::primary_key(),
            placeholders,
            columns,
            marker: PhantomData,
        }
    }
}

impl<T: FromRow> ToSql for Insert<T> {
    fn to_sql(&self) -> String {
        let columns = self
            .columns
            .iter()
            .map(|c| format!(r#""{}""#, c.escape()))
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
