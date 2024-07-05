use crate::model::{
    filter::{Filter, JoinOp},
    Column, Columns, Escape, FromRow, Limit, OrderBy, Placeholders, ToSql, ToValue, Value,
    WhereClause,
};

use std::marker::PhantomData;
use std::ops::Deref;

#[derive(Debug, Default)]
pub struct Select<T: FromRow + ?Sized> {
    pub table_name: String,
    pub primary_key: String,
    pub columns: Columns,
    pub order_by: OrderBy,
    pub limit: Limit,
    pub placeholders: Placeholders,
    pub where_clause: WhereClause,
    _phantom: PhantomData<T>,
}

impl<T: FromRow> Select<T> {
    pub fn new(table_name: &str, primary_key: &str) -> Self {
        Self {
            table_name: table_name.to_string(),
            primary_key: primary_key.to_string(),
            columns: Columns::default(),
            order_by: OrderBy::default(),
            limit: Limit::default(),
            placeholders: Placeholders::default(),
            where_clause: WhereClause::default(),
            _phantom: PhantomData,
        }
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Limit::new(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.limit = self.limit.offset(offset);
        self
    }

    pub fn order_by(mut self, order_by: OrderBy) -> Self {
        self.order_by = order_by;
        self
    }

    pub fn filter(mut self, filters: impl ToFilterable, join_op: JoinOp, not: bool) -> Self {
        let mut filter = Filter::default();
        let table_name = self.table_name.clone();
        let filters = filters.to_filterable();

        for (column, value) in filters.deref() {
            let column = Column::new(&table_name, &column.to_string().as_str());
            let value = value.to_value();

            let value = match value {
                Value::List(_) => {
                    let placeholder = self.placeholders.add(&value);
                    Value::Record(Box::new(placeholder))
                }

                value => self.placeholders.add(&value),
            };

            if not {
                filter.add_not(column, value);
            } else {
                filter.add(column, value);
            }
        }

        match join_op {
            JoinOp::And => self.where_clause.concat(filter),
            JoinOp::Or => self.where_clause.or(filter),
        };

        self
    }

    pub fn or(mut self, query: Self) -> Self {
        let other_filter = query.where_clause.filter();
        let other_placeholders = query.placeholders;
        self.where_clause.or(query.where_clause.filter());
        self
    }

    pub fn filter_and(mut self, filters: impl ToFilterable) -> Self {
        self = self.filter(filters, JoinOp::And, false);
        self
    }

    pub fn filter_or(mut self, filters: impl ToFilterable) -> Self {
        self = self.filter(filters, JoinOp::Or, false);
        self
    }

    pub fn filter_not(mut self, filters: impl ToFilterable) -> Self {
        self = self.filter(filters, JoinOp::And, true);
        self
    }

    pub fn filter_or_not(mut self, filters: impl ToFilterable) -> Self {
        self = self.filter(filters, JoinOp::Or, true);
        self
    }
}

impl<T: FromRow> ToSql for Select<T> {
    fn to_sql(&self) -> String {
        format!(
            r#"SELECT {} FROM "{}"{}{}{}"#,
            self.columns.to_sql(),
            self.table_name.escape(),
            self.where_clause.to_sql(),
            self.order_by.to_sql(),
            self.limit.to_sql()
        )
    }
}

pub struct Filterable {
    filters: Vec<(String, Value)>,
}

impl std::ops::Deref for Filterable {
    type Target = Vec<(String, Value)>;

    fn deref(&self) -> &Self::Target {
        &self.filters
    }
}

pub trait ToFilterable {
    fn to_filterable(&self) -> Filterable;
}

impl<T: ToString, V: ToValue> ToFilterable for (T, V) {
    fn to_filterable(&self) -> Filterable {
        Filterable {
            filters: vec![(self.0.to_string(), self.1.to_value())],
        }
    }
}

impl<T: ToString, V: ToValue> ToFilterable for &[(T, V)] {
    fn to_filterable(&self) -> Filterable {
        Filterable {
            filters: self
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_value()))
                .collect(),
        }
    }
}
