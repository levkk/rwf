//! Implements the `SELECT` query.
use super::{
    Column, Escape, FilterQuery, FromRow, Model, Placeholders, Select, ToColumn, ToSql, ToValue,
    WhereClause,
};
use crate::model::filter::JoinOp;
use crate::model::select::Op;
use crate::model::temporary::{With, WithQuery};
use std::marker::PhantomData;

#[derive(Debug, Clone, crate::prelude::Deserialize, crate::prelude::Serialize)]
enum InsertValues<T: FromRow> {
    Values(Placeholders, i32),
    Select(Select<T>),
}

impl<T: FromRow> InsertValues<T> {
    pub fn is_select(&self) -> bool {
        match self {
            Self::Select(_) => true,
            _ => false,
        }
    }
}

impl<T: FromRow> AsRef<Placeholders> for InsertValues<T> {
    fn as_ref(&self) -> &Placeholders {
        match &self {
            Self::Values(placeholders, _) => placeholders,
            Self::Select(select) => select.get_placeholders(),
        }
    }
}

impl<T: FromRow> AsMut<Placeholders> for InsertValues<T> {
    fn as_mut(&mut self) -> &mut Placeholders {
        match self {
            Self::Values(placeholders, _) => placeholders,
            Self::Select(select) => select.get_placeholders_mut(),
        }
    }
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

    pub fn is_select(&self) -> bool {
        self.values.is_select()
    }
}

impl<T: FromRow> FilterQuery for Insert<T> {
    fn get_table_name(&self) -> &str {
        self.table_name.as_str()
    }

    fn get_where_clause(&self) -> &WhereClause {
        match &self.values {
            InsertValues::Select(select) => &select.get_where_clause(),
            InsertValues::Values(..) => unimplemented!(
                "FilterQuery is only implemented for INSERT where the source is a SELECT Statement"
            ),
        }
    }

    fn get_placeholders(&self) -> &Placeholders {
        self.values.as_ref()
    }

    fn get_where_clause_mut(&mut self) -> &mut WhereClause {
        match &mut self.values {
            InsertValues::Select(select) => select.get_where_clause_mut(),
            InsertValues::Values(..) => unimplemented!(
                "FilterQuery is only implemented for INSERT where the source is a SELECT Statement"
            ),
        }
    }

    fn get_placeholders_mut(&mut self) -> &mut Placeholders {
        self.values.as_mut()
    }

    fn filter(
        mut self,
        column: impl ToColumn,
        value: impl ToValue,
        join_op: JoinOp,
        op: Op,
    ) -> Self {
        match self.values {
            InsertValues::Values(placeholders, offset) => {
                self.values = InsertValues::Values(placeholders, offset)
            }
            InsertValues::Select(select) => {
                self.values =
                    InsertValues::Select(select.filter_internal(column, value, join_op, op))
            }
        }
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
