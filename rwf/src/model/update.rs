//! Implements the `UPDATE` statement.
use super::{
    Column, Escape, FromRow, Model, Placeholders, Select, ToColumn, ToSql, ToValue, WhereClause,
};
use crate::model::select::FilterQuery;
use crate::model::temporary::{With, WithQuery};
use std::marker::PhantomData;

#[derive(Debug, Clone, crate::prelude::Deserialize, crate::prelude::Serialize)]
pub struct Update<T> {
    table_name: String,
    primary_key: String,
    pub placeholders: Placeholders,
    columns: Vec<Column>,
    where_clause: WhereClause,
    marker: PhantomData<T>,
    with: With,
    using: Vec<String>,
}

impl<T: Model> Update<T> {
    pub fn empty() -> Self {
        Self {
            table_name: T::table_name().to_string(),
            primary_key: T::primary_key().to_string(),
            placeholders: Placeholders::new(),
            columns: vec![],
            where_clause: WhereClause::default(),
            marker: PhantomData,
            with: With::default(),
            using: vec![],
        }
    }

    pub fn new(model: T) -> Self {
        let columns = T::column_names();
        let values = model.values();
        Self::from_columns(model.id(), columns, &values)
    }

    /// Create an update query for specific columns and values only.
    pub fn from_columns(
        id: impl ToValue,
        columns: &[impl ToColumn],
        values: &[impl ToValue],
    ) -> Self {
        let mut update = Self::empty();

        // Add the primary key selector.
        let id_placeholder = update.placeholders.add(&id.to_value());
        update
            .where_clause
            .add(Column::name(&update.primary_key), id_placeholder);

        update.columns(columns, values)
    }

    pub fn columns(mut self, columns: &[impl ToColumn], values: &[impl ToValue]) -> Self {
        for (column, value) in columns.iter().zip(values.iter()) {
            self.columns.push(column.to_column());
            self.placeholders.add(&value.to_value());
        }
        self
    }
}

impl<T: FromRow> FilterQuery for Update<T> {
    fn get_table_name(&self) -> &str {
        self.table_name.as_str()
    }

    fn get_where_clause(&self) -> &WhereClause {
        &self.where_clause
    }

    fn get_placeholders(&self) -> &Placeholders {
        &self.placeholders
    }

    fn get_where_clause_mut(&mut self) -> &mut WhereClause {
        &mut self.where_clause
    }

    fn get_placeholders_mut(&mut self) -> &mut Placeholders {
        &mut self.placeholders
    }
}

impl<T: FromRow> WithQuery for Update<T> {
    fn with_statements(&self) -> &With {
        &self.with
    }

    fn with_statements_mut(&mut self) -> &mut With {
        &mut self.with
    }

    fn get_statement_offset(&self) -> i32 {
        (self.where_clause.placeholders() + self.columns.len()) as i32
    }

    fn add_offset(&mut self, offset: i32) {
        self.where_clause.add_offset(offset);
    }
    fn placeholders(&self) -> Placeholders {
        let mut placeholders = self.with.placeholders();
        placeholders.push(self.placeholders.clone());
        Placeholders::from_iter(placeholders)
    }
}

impl<T: Model> From<Select<T>> for Update<T> {
    fn from(select: Select<T>) -> Update<T> {
        let mut update = Update::empty();
        update.where_clause = select.where_clause;
        update.placeholders = select.placeholders;
        update.with = select.with;
        update
    }
}

impl<T: FromRow> ToSql for Update<T> {
    fn to_sql(&self) -> String {
        let where_placeholders = self.where_clause.placeholders() + self.get_with_offset() as usize;
        let sets = self
            .columns
            .iter()
            .enumerate()
            .map(|(idx, column)| {
                format!(r#"{} = ${}"#, column.to_sql(), idx + where_placeholders + 1)
            })
            .collect::<Vec<_>>()
            .join(", ");

        let using = if self.using.is_empty() {
            String::new()
        } else {
            format!(
                r#" FROM {} "#,
                self.using
                    .iter()
                    .map(|s| format!(r#""{}""#, s.escape()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        format!(
            r#"{}UPDATE "{}" SET {}{}{} RETURNING *"#,
            self.with.to_sql(),
            self.table_name.escape(),
            sets,
            using,
            self.where_clause.to_sql(),
        )
    }
}
