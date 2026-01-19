//! Implements the `SELECT` query.
use crate::model::{
    column::ToColumn,
    filter::{Filter, JoinOp},
    Column, Columns, Escape, FromRow, Join, Joins, Limit, Lock, OrderBy, Placeholders, Query,
    ToSql, ToValue, Value, WhereClause,
};

use crate::model::combine::{Combine, Combines};
use crate::model::temporary::{With, WithQuery};
use std::marker::PhantomData;

#[derive(PartialEq, Debug, Clone, Copy, Eq, Ord, PartialOrd)]
pub enum Op {
    Equals,
    NotEquals,
    LesserThan,
    GreaterThan,
    GreaterEqualThan,
    LesserEqualThan,
}

#[derive(Debug, Default, Clone, crate::prelude::Deserialize, crate::prelude::Serialize)]
pub struct Select<T: FromRow + ?Sized> {
    pub table_name: String,
    pub primary_key: String,
    pub columns: Columns,
    pub order_by: OrderBy,
    pub limit: Limit,
    pub placeholders: Placeholders,
    pub where_clause: WhereClause,
    pub joins: Joins,
    pub(super) lock: Lock,
    pub(super) group: bool,
    pub(super) combines: Combines<T>,
    pub(super) with: With,
    _phantom: PhantomData<T>,
}

impl<T: FromRow> Select<T> {
    /// Create new SELECT query against the table with the given primary key.
    pub fn new(table_name: &str, primary_key: &str) -> Self {
        Self {
            table_name: table_name.to_string(),
            primary_key: primary_key.to_string(),
            columns: Columns::default(),
            order_by: OrderBy::default(),
            limit: Limit::default(),
            placeholders: Placeholders::default(),
            where_clause: WhereClause::default(),
            joins: Joins::default(),
            lock: Lock::default(),
            group: false,
            combines: Combines::default(),
            with: With::default(),
            _phantom: PhantomData,
        }
    }

    /// Add a LIMIT to the query.
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Limit::new(limit);
        self
    }

    /// Add an OFFSET.
    pub fn offset(mut self, offset: i64) -> Self {
        self.limit = self.limit.offset(offset);
        self
    }

    /// Add an ORDER BY clause.
    pub fn order_by(mut self, order_by: OrderBy) -> Self {
        self.order_by = order_by;
        self
    }

    pub fn lock(mut self) -> Self {
        self.lock = Lock::new();
        self
    }

    pub fn skip_locked(mut self) -> Self {
        self.lock = self.lock.skip_locked();
        self
    }

    pub fn exists(mut self) -> Self {
        self.columns = self.columns.exists();
        self
    }

    pub fn join(mut self, join: Join) -> Self {
        self.joins = self.joins.add(join);
        self.columns = self.columns.table_name(&self.table_name);
        self
    }

    pub fn get_joins(&self) -> Joins {
        self.joins.clone()
    }

    pub fn add_joins(mut self, joins: Joins) -> Self {
        for join in joins.joins() {
            self.joins = self.joins.add(join.clone());
        }

        self
    }

    pub fn insert_columns(&self) -> (Vec<Column>, Vec<Value>) {
        let (columns, values) = self.where_clause.insert_columns();
        let mut actual_values = vec![];

        for value in values {
            let value = match value {
                Value::Placeholder(id) => self
                    .placeholders
                    .get(id)
                    .expect("to have a valid placeholder")
                    .clone(),
                value => value,
            };

            actual_values.push(value);
        }

        (columns, actual_values)
    }

    pub fn or(&self) -> Self {
        let mut select = Select::new(&self.table_name, &self.primary_key);
        select.placeholders = self.placeholders.clone();
        select
    }

    pub fn select_additional(mut self, column: impl ToColumn) -> Self {
        self.columns = self.columns.add_column(column);
        self
    }

    pub fn group(mut self, columns: &[impl ToColumn]) -> Self {
        self.group = true;
        self.columns = Columns::pick(columns);
        self
    }

    pub fn count(mut self) -> Self {
        self.columns = self.columns.count();
        self
    }

    fn combine(mut self, mut other: Combine<T>) -> Self {
        let withs = other.take_with();
        let with_offset = self.get_with_offset();
        if !withs.is_empty() {
            let offset = self.with.extend(withs);
            self.add_offset(offset);
        }
        other.add_offset(self.get_statement_offset() + with_offset);
        self.combines.add_query(other);
        self
    }
    pub fn add_union(self, other: Query<T>) -> Self {
        if let Ok(q) = Combine::union(other) {
            self.combine(q)
        } else {
            self
        }
    }
    pub fn add_union_all(self, other: Query<T>) -> Self {
        if let Ok(q) = Combine::union_all(other) {
            self.combine(q)
        } else {
            self
        }
    }
    pub fn add_intersect(self, other: Query<T>) -> Self {
        if let Ok(q) = Combine::intersect(other) {
            self.combine(q)
        } else {
            self
        }
    }
    pub fn add_intersect_all(self, other: Query<T>) -> Self {
        if let Ok(q) = Combine::intersect_all(other) {
            self.combine(q)
        } else {
            self
        }
    }
    pub fn add_except(self, other: Query<T>) -> Self {
        if let Ok(q) = Combine::except(other) {
            self.combine(q)
        } else {
            self
        }
    }
    pub fn add_except_all(self, other: Query<T>) -> Self {
        if let Ok(q) = Combine::except_all(other) {
            self.combine(q)
        } else {
            self
        }
    }
}

impl<T: FromRow> WithQuery for Select<T> {
    fn with_statements(&self) -> &With {
        &self.with
    }
    fn with_statements_mut(&mut self) -> &mut With {
        &mut self.with
    }

    fn get_statement_offset(&self) -> i32 {
        self.combines.placeholders_id() + self.where_clause.placeholders() as i32
    }

    fn add_offset(&mut self, offset: i32) {
        self.where_clause.add_offset(offset);
        self.combines.add_offset(offset);
    }
    fn placeholders(&self) -> Placeholders {
        if self.combines.is_empty() && self.with.is_empty() {
            self.placeholders.clone()
        } else {
            let mut placeholders = vec![];
            placeholders.extend(self.with.placeholders());
            placeholders.push(self.placeholders.clone());
            placeholders.extend(self.combines.placeholders());
            Placeholders::from_iter(placeholders)
        }
    }
}

impl<T: FromRow> ToSql for Select<T> {
    fn to_sql(&self) -> String {
        let group = if self.group {
            format!("GROUP BY {} ", self.columns.to_sql())
        } else {
            "".to_string()
        };
        format!(
            r#"{}{}SELECT {} FROM "{}"{}{}{}{}{}{}{}{}"#,
            self.with.to_sql(),
            {
                if !self.combines.is_empty() {
                    "("
                } else {
                    ""
                }
            },
            self.columns.to_sql(),
            self.table_name.escape(),
            self.joins.to_sql(),
            self.where_clause.to_sql(),
            group,
            self.order_by.to_sql(),
            self.limit.to_sql(),
            self.lock.to_sql(),
            {
                if !self.combines.is_empty() {
                    ")"
                } else {
                    ""
                }
            },
            self.combines.to_sql()
        )
    }
}

pub trait FilterQuery: Sized {
    fn get_table_name(&self) -> &str;
    fn get_where_clause(&self) -> &WhereClause;
    fn get_placeholders(&self) -> &Placeholders;
    fn get_where_clause_mut(&mut self) -> &mut WhereClause;
    fn get_placeholders_mut(&mut self) -> &mut Placeholders;

    fn filter(self, column: impl ToColumn, value: impl ToValue, join_op: JoinOp, op: Op) -> Self {
        self.filter_internal(column, value, join_op, op)
    }
    fn filter_internal(
        mut self,
        column: impl ToColumn,
        value: impl ToValue,
        join_op: JoinOp,
        op: Op,
    ) -> Self {
        let mut filter = Filter::default();

        let column = {
            let column = column.to_column();
            if !column.qualified() {
                column.qualify(self.get_table_name())
            } else {
                column
            }
        };

        let value = value.to_value();

        // Null is handled by the filter.
        let value = if !value.is_null() {
            match value {
                Value::List(_) => {
                    let placeholder = self.get_placeholders_mut().add(&value);
                    Value::Record(Box::new(placeholder))
                }

                Value::Column(ref _column) => value,
                Value::Function(ref _function) => value,

                value => self.get_placeholders_mut().add(&value),
            }
        } else {
            value
        };

        match op {
            Op::Equals => filter.add(column, value),
            Op::NotEquals => filter.add_not(column, value),
            Op::LesserThan => filter.lt(column, value),
            Op::GreaterThan => filter.gt(column, value),
            Op::GreaterEqualThan => filter.gte(column, value),
            Op::LesserEqualThan => filter.lte(column, value),
        }

        match join_op {
            JoinOp::And => self.get_where_clause_mut().concat(filter),
            JoinOp::Or => self.get_where_clause_mut().or(filter),
        };
        self
    }

    fn filter_and(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::And, Op::Equals);
        self
    }

    fn filter_or(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::Or, Op::Equals);
        self
    }

    fn filter_not(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::And, Op::NotEquals);
        self
    }

    fn filter_or_not(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::Or, Op::NotEquals);
        self
    }

    fn filter_lt(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::And, Op::LesserThan);
        self
    }

    fn filter_gt(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::And, Op::GreaterThan);
        self
    }

    fn filter_gte(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::And, Op::GreaterEqualThan);
        self
    }

    fn filter_lte(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::And, Op::LesserEqualThan);
        self
    }
}

impl<T: FromRow> FilterQuery for Select<T> {
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

    fn filter(
        mut self,
        column: impl ToColumn,
        value: impl ToValue,
        join_op: JoinOp,
        op: Op,
    ) -> Self {
        let placeholder_id = self.get_where_clause().placeholders();
        self = self.filter_internal(column, value, join_op, op);
        if self.where_clause.placeholders() > placeholder_id {
            self.combines.inc_placeholders()
        }
        self
    }
}
