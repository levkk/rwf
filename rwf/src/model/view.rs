use std::fmt::write;

use crate::model::{Error, Escape, ToColumn, ToSql};
use crate::model::column::ToAggregation;

use super::select::Select;
use super::Model;
use super::join::{Join, Joins, Association};
use super::picked::Picked;

#[derive(Debug,Clone)]
pub struct View<T> where T: Model {
    pivot: Picked<T>
}

impl<T> From<Picked<T>> for View<T> where T: Model {
    fn from(value: Picked<T>) -> Self {
        Self{pivot: value}
    }
}

impl<T> View<T> where T: Model {
    pub fn new(select: Select<T>) -> Self {Self{pivot: Picked::from(select)}}
    pub fn add_aggregated_column(mut self, column: impl ToColumn, agg: impl ToAggregation, alias: Option<impl ToString>) -> Self {
        let col = column.to_column().qualify(self.pivot.select.table_name.clone());
        let piv = self.pivot.add_column(if let Some(alias) = alias {col.alias(alias)} else {col}, agg);
        Self::from(piv)
    }
    pub fn add_view_column(mut self, column: impl ToColumn) -> Self {
        let alias: Option<String> = None;
        self.add_aggregated_column(column, "", alias)
    }
    pub fn join<U: Association<T>>(mut self, other:View<U>) -> Self {
        self.pivot.select.joins = self.pivot.select.joins.add(U::construct_join());
        Self::from(self.pivot.merge(other.pivot.columns()))
    }
    pub fn from_row(&self, row: tokio_postgres::Row) -> Result<Self, Error> {
        self.pivot.from_row(row).map(|pick| Self::from(pick))
    }
    pub fn create_view(&self, name: impl ToString) -> String {
        format!(r#"CREATE VIEW "{}" AS ({})"#, name.to_string().escape(), self)
    }
}

impl<T> ToSql for View<T> where T: Model {
    fn to_sql(&self) -> String {
        self.pivot.to_sql()
    }
}

impl<T> std::fmt::Display for View<T> where T: Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_sql())
    }
}


