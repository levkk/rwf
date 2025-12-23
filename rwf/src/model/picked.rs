//! Select only a few columns.
#![allow(dead_code, unused_variables)]

use std::collections::HashMap;

use crate::model::column::{ToAggregation};

use super::*;

#[derive(Debug, Clone)]
pub struct Picked<T: FromRow> {
    pub select: Select<T>,
    columns: Vec<Column>,
    data: Vec<Value>
}

impl<T: FromRow> Picked<T> {
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

    pub fn map(self) -> HashMap<Column, Value> {
        self.columns.into_iter().zip(self.data.into_iter()).collect()
    }

    pub fn get_entry(&self, alias: impl ToString) -> Option<(&Column, &Value)> {
        let alias = alias.to_string();
        self.columns.iter().zip(self.data.iter()).find(|(c,v)| alias.eq(c.get_alias()))
    }

    pub fn add_column(mut self, column: impl ToColumn, agg: impl ToAggregation) -> Self {
        let column = column.to_column();
        let column =  if !column.qualified() {column.qualify(self.select.table_name.as_str())} else {column};
        self.columns.push(column.agg(agg));
        self
    }

    pub(super) fn from_row(&self, row: tokio_postgres::Row) -> Result<Self, Error> {
        let mut data = Vec::new();
        for col in self.columns.iter() {
            data.push(row.try_get(col.get_name())?);
        }
        Ok(Self{select: self.select.clone(), columns: self.columns.clone(), data})
    }
}

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

impl<T: FromRow> From<Select<T>> for Picked<T> {
    fn from(value: Select<T>) -> Self {
        let columns = if value.group {
            value.columns.columns.iter().map(|col| col.to_column().agg("")).collect::<Vec<Column>>()
        } else {Vec::new()};
        Self {select: value, columns, data: Vec::new()}
    }
}
