use crate::model::ToColumn;
use crate::model::column::ToAggregation;

use super::select::Select;
use super::Model;
use super::join::{Join, Joins, Association};
use super::picked::Picked;

#[derive(Debug,Clone)]
struct View<T> where T: Model {
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
    pub fn join<U: Association<T>>(self, other:View<U>) -> Self {
        T::join::<U>().into().joins().into_iter().for_each(|j| {
            self.pivot.select.joins.add(j.clone());
        });
        Self::from(self.pivot.merge(other.pivot.columns()))


    }
}
