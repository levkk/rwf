use crate::model::{
    column::ToColumn,
    filter::{Filter, JoinOp},
    Column, Columns, Escape, FromRow, Join, Joins, Limit, Lock, OrderBy, Placeholders, ToSql,
    ToValue, Value, WhereClause,
};

use std::marker::PhantomData;

#[derive(PartialEq, Debug)]
enum Op {
    Equals,
    NotEquals,
    LesserThan,
    GreaterThan,
    GreaterEqualThan,
    LesserEqualThan,
}

#[derive(Debug, Default, Clone)]
pub struct Select<T: FromRow + ?Sized> {
    pub table_name: String,
    pub primary_key: String,
    pub columns: Columns,
    pub order_by: OrderBy,
    pub limit: Limit,
    pub placeholders: Placeholders,
    pub where_clause: WhereClause,
    pub joins: Joins,
    lock: Lock,
    group: bool,
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

    fn filter(
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
                column.qualify(&self.table_name)
            } else {
                column
            }
        };

        let value = value.to_value();

        // Null is handled by the filter.
        let value = if !value.is_null() {
            match value {
                Value::List(_) => {
                    let placeholder = self.placeholders.add(&value);
                    Value::Record(Box::new(placeholder))
                }

                Value::Column(ref _column) => value,
                Value::Function(ref _function) => value,

                value => self.placeholders.add(&value),
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
            JoinOp::And => self.where_clause.concat(filter),
            JoinOp::Or => self.where_clause.or(filter),
        };

        self
    }

    pub fn filter_and(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::And, Op::Equals);
        self
    }

    pub fn filter_or(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::Or, Op::Equals);
        self
    }

    pub fn filter_not(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::And, Op::NotEquals);
        self
    }

    pub fn filter_or_not(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::Or, Op::NotEquals);
        self
    }

    pub fn filter_lt(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::And, Op::LesserThan);
        self
    }

    pub fn filter_gt(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::And, Op::GreaterThan);
        self
    }

    pub fn filter_gte(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::And, Op::GreaterEqualThan);
        self
    }

    pub fn filter_lte(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::And, Op::LesserEqualThan);
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

    pub fn placeholders(&self) -> &Placeholders {
        &self.placeholders
    }

    pub fn where_clause(&self) -> &WhereClause {
        &self.where_clause
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
}

impl<T: FromRow> ToSql for Select<T> {
    fn to_sql(&self) -> String {
        let group = if self.group {
            format!("GROUP BY {} ", self.columns.to_sql())
        } else {
            "".to_string()
        };
        format!(
            r#"SELECT {} FROM "{}"{}{}{}{}{}{}"#,
            self.columns.to_sql(),
            self.table_name.escape(),
            self.joins.to_sql(),
            self.where_clause.to_sql(),
            group,
            self.order_by.to_sql(),
            self.limit.to_sql(),
            self.lock.to_sql(),
        )
    }
}
