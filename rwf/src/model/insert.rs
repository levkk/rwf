//! Implements the `SELECT` query.
use super::{Column, Escape, FromRow, Model, Placeholders, Select, ToColumn, ToSql, ToValue};
use crate::model::temporary::{With, WithQuery};
use std::marker::PhantomData;

#[derive(Debug, Clone, crate::prelude::Deserialize, crate::prelude::Serialize)]
enum InsertValues<T: FromRow> {
    Values(Placeholders, i32),
    Select(Select<T>),
}

#[derive(Debug, Clone, crate::prelude::Deserialize, crate::prelude::Serialize)]
pub struct Insert<T: FromRow + ?Sized> {
    table_name: String,
    columns: Vec<Column>,
    values: InsertValues<T>,
    marker: PhantomData<T>,
    no_conflict: bool,
    unique_by: Vec<Column>,
    with: With,
}

impl<T: Model> Insert<T> {
    pub fn new(model: T) -> Self {
        let columns = T::column_names().iter().map(Column::name).collect();
        let values = model.values();
        let mut placeholders = Placeholders::new();
        for value in values {
            placeholders.add(&value);
        }

        Self {
            table_name: T::table_name().to_string(),
            values: InsertValues::Values(placeholders, 0),
            columns,
            marker: PhantomData,
            no_conflict: false,
            unique_by: vec![],
            with: With::default(),
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
            values: InsertValues::Values(placeholders, 0),
            marker: PhantomData,
            no_conflict: false,
            unique_by: vec![],
            with: With::default(),
        }
    }

    pub fn no_conflict(mut self) -> Self {
        self.no_conflict = true;
        self
    }

    pub fn unique_by(mut self, columns: &[impl ToColumn]) -> Self {
        self.unique_by = columns.iter().map(|c| c.to_column()).collect();
        self
    }
}

impl<T: FromRow> WithQuery for Insert<T> {
    fn with_statements(&self) -> &With {
        &self.with
    }

    fn with_statements_mut(&mut self) -> &mut With {
        &mut self.with
    }

    fn get_statement_offset(&self) -> i32 {
        match &self.values {
            InsertValues::Values(..) => self.columns.len() as i32,
            InsertValues::Select(select) => select.get_statement_offset(),
        }
    }

    fn add_offset(&mut self, offset: i32) {
        match &mut self.values {
            InsertValues::Values(_, value_offset) => *value_offset += offset,
            InsertValues::Select(select) => select.add_offset(offset),
        }
    }

    fn placeholders(&self) -> Placeholders {
        let mut placeholders = self.with.placeholders();
        placeholders.push(match &self.values {
            InsertValues::Values(placeholders, _) => placeholders.clone(),
            InsertValues::Select(select) => select.placeholders(),
        });
        Placeholders::from_iter(placeholders)
    }
}

impl<T: Model> From<Select<T>> for Insert<T> {
    fn from(mut value: Select<T>) -> Self {
        let with = std::mem::take(value.with_statements_mut());
        let values = InsertValues::Select(value);
        Self {
            table_name: T::table_name().to_string(),
            columns: T::column_names().iter().map(Column::name).collect(),
            values,
            marker: Default::default(),
            no_conflict: false,
            unique_by: vec![],
            with,
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

        let no_conflict = if self.no_conflict {
            "ON CONFLICT DO NOTHING ".to_string()
        } else if !self.unique_by.is_empty() {
            let columns = self
                .unique_by
                .clone()
                .into_iter()
                .map(|c| c.unqualify())
                .collect::<Vec<_>>();
            let on_conflict = columns
                .iter()
                .map(|c| c.to_sql())
                .collect::<Vec<_>>()
                .join(", ");
            let update = columns
                .iter()
                .map(|c| format!("{} = EXCLUDED.{}", c.to_sql(), c.to_sql()))
                .collect::<Vec<_>>()
                .join(", ");
            format!("ON CONFLICT ({}) DO UPDATE SET {} ", on_conflict, update)
        } else {
            "".to_string()
        };

        let values = match &self.values {
            InsertValues::Select(select) => select.to_sql(),
            InsertValues::Values(_, offset) => {
                let placeholders = self
                    .columns
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format!("${}", i as i32 + 1 + offset + self.get_with_offset()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("VALUES ({})", placeholders)
            }
        };
        format!(
            r#"{}INSERT INTO "{}" ({}) {} {}RETURNING *"#,
            self.with.to_sql(),
            self.table_name.escape(),
            columns,
            values,
            no_conflict,
        )
    }
}
