//! Select only a few columns.
#![allow(dead_code, unused_variables)]

use std::collections::HashMap;

use crate::model::column::{ToAggregation};

use super::*;

/// A Struct extending a SELECT Query with the ability to return only specified columns as well as
/// aggregated ones. So it makes Queries like 'SELECT table.column FROM table' possible as well as
/// 'SELECT SUM(table.column) FROM table'
///
/// Only the 'SELECT' Part is implemented by `Picked<T>` Everything else (FROM, JOIN, WHERE, GROUP) is
/// taken from the underlaying `Select<T>`
#[derive(Debug, Clone)]
pub struct Picked<T: FromRow> {
    pub select: Select<T>,
    columns: Vec<Column>,
    data: Vec<Value>
}

impl<T: FromRow> Picked<T> {
    /// Extend the 'GROUP BY' Part of a Query by some Columns as well as adding them to the 'SELECT' part.
    pub fn group(mut self, group: &[impl ToColumn]) -> Self {
        self.columns.extend(group.iter().map(|col| {
            let column = col.to_column();
            if !column.qualified() {
                column.qualify(self.select.table_name.as_str())
            } else { column }.agg("")
        }));
        self.select = self.select.group(group);
        self
    }

    /// Call on the OK Result of `Self::from_row(&self, row)` to get a HashMap<Column, Value>
    /// containing the Query Result in a convinient Form.
    pub fn map(self) -> HashMap<Column, Value> {
        self.columns.into_iter().zip(self.data.into_iter()).collect()
    }
    pub fn columns(self) -> Vec<Column> {self.columns}
    pub fn merge(mut self, mut columns: Vec<Column>) -> Self {self.columns.append(&mut columns); self}

    /// Call on the Ok Result of `Self::from_row(&self, row)` to get a `Column` by its alias as well
    /// as the `Value` belonging to it.
    pub fn get_entry(&self, alias: impl ToString) -> Option<(&Column, &Value)> {
        let alias = alias.to_string();
        self.columns.iter().zip(self.data.iter()).find(|(c,v)| alias.eq(c.get_alias()))
    }


    /// Add a Column to the Select Part  of the Query and ensure it's Aggregation and Alias is set.
    /// eg. `Self::add_column(self, "id", "", None)` results in 'SELECT id as id'
    /// where as `Self::add_column(self, "id", "MAX", "max_id")` results in 'SELECT MAX(id) as
    /// max_id'
    pub fn add_column(mut self, column: impl ToColumn, agg: impl ToAggregation, alias: Option<impl ToString>) -> Self {
        let column = column.to_column();
        let column =  if !column.qualified() {column.qualify(self.select.table_name.as_str())} else {column};
        let column = if let Some(alias) = alias {column.alias(alias)} else {column};
        self.columns.push(column.agg(agg));
        self
    }
    /// Destroy self for an `Vec<Column>` hold by.
    pub fn columns(self) -> Vec<Column> {self.columns}

    /// Extend the columns hold by an `Vec<Column>`
    pub fn merge(mut self, mut columns: Vec<Column>) -> Self {
        self.columns.append(&mut columns);
        self
    }

    /// Workaround because `FromRow::from_row` accept no `&self` Parameter
    pub(super) fn from_row(&self, row: tokio_postgres::Row) -> Result<Self, Error> {
        let mut data = Vec::new();
        for col in self.columns.iter() {
            data.push(row.try_get(col.get_alias())?);
        }
        Ok(Self{select: self.select.clone(), columns: self.columns.clone(), data})
    }
}

/// Impl ToSql taking all except SELECT from underlaying `Select<T>`
impl<T: FromRow> ToSql for Picked<T> {

    fn to_sql(&self) -> String {
        let group = if self.select.group {
            format!(" GROUP BY {} ", self.select.columns.to_sql())
        } else {
            "".to_string()
        };

        let columns = self.columns.iter().map(|col| col.to_sql()).collect::<Vec<_>>().join(", ");
        format!(
            r#"SELECT {} FROM "{}"{}{}{}{}{}{}"#,
            columns,
            self.select.table_name.escape(),
            self.select.joins.to_sql(),
            self.select.where_clause.to_sql(),
            group,
            self.select.order_by.to_sql(),
            self.select.limit.to_sql(),
            self.select.lock.to_sql(),
        )
    }
}

/// Default way to create an Pickex Query, as `Select<T>` is the only required component. 
/// Includes all columns included in the GROUP BY statement in the SELECT statement
impl<T: FromRow> From<Select<T>> for Picked<T> {
    fn from(value: Select<T>) -> Self {
        let columns = if value.group {
            value.columns.columns.iter().map(|col| col.to_column().agg("")).collect::<Vec<Column>>()
        } else {Vec::new()};
        Self {select: value, columns, data: Vec::new()}
    }
}
