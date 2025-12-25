#![allow(dead_code, unused_variables)]
use crate::model::column::ToAggregation;
use crate::model::{Error, Escape, ToColumn, ToSql};

use super::join::Association;
use super::picked::Picked;
use super::select::Select;
use super::{Model, Query};

/// Implements a joined View with Picked Columns
#[derive(Debug, Clone)]
pub struct View<T>
where
    T: Model,
{
    pivot: Picked<T>,
}

/// Default way to create a new one, is from a Picked one
impl<T> From<Picked<T>> for View<T>
where
    T: Model,
{
    fn from(value: Picked<T>) -> Self {
        Self { pivot: value }
    }
}

/// For the sake of simplicity, a wrapper from a qualified Query
impl<T> TryFrom<Query<T>> for View<T>
where
    T: Model,
{
    type Error = &'static str;
    fn try_from(value: Query<T>) -> Result<Self, Self::Error> {
        if let Query::Picked(picked) = value {
            Ok(Self::from(picked))
        } else if let Query::Select(select) = value {
            Ok(Self::new(select))
        } else {
            Err("Only implemended for Picked and Select Querys")
        }
    }
}

/// Convert into a Query
impl<T> Into<Query<T>> for View<T>
where
    T: Model,
{
    fn into(self) -> Query<T> {
        Query::Picked(self.pivot)
    }
}

impl<T> View<T>
where
    T: Model,
{
    /// Create a new View from a Select Query
    pub fn new(select: Select<T>) -> Self {
        Self {
            pivot: Picked::from(select),
        }
    }
    /// Add a Column to tge view with Aggregation and alias
    pub fn add_aggregated_column(
        self,
        column: impl ToColumn,
        agg: impl ToAggregation,
        alias: Option<impl ToString>,
    ) -> Self {
        let col = column
            .to_column()
            .qualify(self.pivot.select.table_name.clone());
        let piv = self.pivot.add_column(column, agg, alias);
        Self::from(piv)
    }
    /// Add a Column without Aggregation and alias
    pub fn add_view_column(self, column: impl ToColumn, alias: Option<impl ToString>) -> Self {
        self.add_aggregated_column(column, "", alias)
    }
    /// Join with another view including all Picked Columns
    pub fn join<U: Association<T>>(mut self, other: View<U>) -> Self {
        self.pivot.select.joins = self.pivot.select.joins.add(U::construct_join());
        Self::from(self.pivot.merge(other.pivot.columns()))
    }
    /// Workaround as FromRow is only implementable for dedicated Structs
    pub fn from_row(&self, row: tokio_postgres::Row) -> Result<Self, Error> {
        self.pivot.from_row(row).map(|pick| Self::from(pick))
    }
    /// For later Usage. Intended is a Macro which creates a Struct related to the Joined Query
    /// implementing the Model Trait
    pub fn create_view(&self, name: impl ToString) -> String {
        format!(
            r#"CREATE VIEW "{}" AS ({})"#,
            name.to_string().escape(),
            self
        )
    }
    /// Use all columns including the primary key from the main Table
    pub fn use_all_pivot() -> Self {
        Self::try_from(
            T::all()
                .select_columns(&["id"])
                .select_columns(T::column_names()),
        )
        .unwrap()
    }
    /// Use all columns excluding the primary Key. used for the joined Table
    pub fn use_all() -> Self {
        Self::try_from(T::all().select_columns(T::column_names())).unwrap()
    }
}

/// Deref ToSql to Picked
impl<T> ToSql for View<T>
where
    T: Model,
{
    fn to_sql(&self) -> String {
        self.pivot.to_sql()
    }
}

/// Deref Display to Picked
impl<T> std::fmt::Display for View<T>
where
    T: Model,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_sql())
    }
}
