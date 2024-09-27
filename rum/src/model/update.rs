use super::{
    Column, Escape, FromRow, Model, Placeholders, Select, ToColumn, ToSql, ToValue, WhereClause,
};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct Update<T> {
    table_name: String,
    primary_key: String,
    pub placeholders: Placeholders,
    columns: Vec<Column>,
    where_clause: WhereClause,
    marker: PhantomData<T>,
}

impl<T: Model> Update<T> {
    pub fn empty() -> Self {
        Self {
            table_name: T::table_name(),
            primary_key: T::primary_key(),
            placeholders: Placeholders::new(),
            columns: vec![],
            where_clause: WhereClause::default(),
            marker: PhantomData,
        }
    }

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
        let mut update = Self::empty().columns(columns, values);

        // Add the primary key selector.
        let id_placeholder = update.placeholders.add(&id.to_value());
        update
            .where_clause
            .add(Column::name(&update.primary_key), id_placeholder);

        update
    }

    pub fn columns(mut self, columns: &[impl ToColumn], values: &[impl ToValue]) -> Self {
        for (column, value) in columns.iter().zip(values.iter()) {
            self.columns.push(column.to_column());
            self.placeholders.add(&value.to_value());
        }
        self
    }
}

impl<T: Model> From<Select<T>> for Update<T> {
    fn from(select: Select<T>) -> Update<T> {
        let mut update = Update::empty();
        update.where_clause = select.where_clause;
        update.placeholders = select.placeholders;

        update
    }
}

impl<T: FromRow> ToSql for Update<T> {
    fn to_sql(&self) -> String {
        let sets = self
            .columns
            .iter()
            .enumerate()
            .map(|(idx, column)| format!(r#"{} = ${}"#, column.to_sql(), idx + 1))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            r#"UPDATE "{}" SET {} {} RETURNING *"#,
            self.table_name.escape(),
            sets,
            self.where_clause.to_sql(),
        )
    }
}
