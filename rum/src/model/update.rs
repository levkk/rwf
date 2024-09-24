use super::{Escape, FromRow, Model, Placeholders, ToColumn, ToSql, ToValue, Value};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Update<T> {
    table_name: String,
    primary_key: String,
    pub placeholders: Placeholders,
    columns: Vec<String>,
    marker: PhantomData<T>,
}

impl<T: Model> Update<T> {
    pub fn new(model: T) -> Self {
        let columns = T::column_names();
        let values = model.values();
        Self::from_columns(model.id(), &columns, &values)
    }

    /// Create an update query for specific columns and values only.
    pub fn from_columns(
        id: impl ToValue,
        columns: &[impl ToColumn],
        values: &[impl ToValue],
    ) -> Self {
        let columns = columns
            .iter()
            .map(|c| c.to_column().to_string())
            .collect::<Vec<_>>();
        let values = values.iter().map(|v| v.to_value()).collect::<Vec<_>>();
        let mut placeholders = Placeholders::new();
        for value in values {
            placeholders.add(&value);
        }
        placeholders.add(&id.to_value());

        Self {
            table_name: T::table_name(),
            primary_key: T::primary_key(),
            placeholders,
            columns,
            marker: PhantomData,
        }
    }
}

impl<T: FromRow> ToSql for Update<T> {
    fn to_sql(&self) -> String {
        let where_id = format!(
            r#""{}" = ${}"#,
            self.primary_key,
            self.placeholders.id() - 1
        );
        let sets = self
            .columns
            .iter()
            .enumerate()
            .map(|(idx, column)| format!(r#"{} = ${}"#, column, idx + 1))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            r#"UPDATE "{}" SET {} WHERE {} RETURNING *"#,
            self.table_name.escape(),
            sets,
            where_id
        )
    }
}
