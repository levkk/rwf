use crate::model::{
    column::ToColumn,
    filter::{Filter, JoinOp},
    Columns, Escape, FromRow, Join, Joins, Limit, OrderBy, Placeholders, ToSql, ToValue, Value,
    WhereClause,
};

use std::marker::PhantomData;

#[derive(Debug, Default)]
pub struct Select<T: FromRow + ?Sized> {
    pub table_name: String,
    pub primary_key: String,
    pub columns: Columns,
    pub order_by: OrderBy,
    pub limit: Limit,
    pub placeholders: Placeholders,
    pub where_clause: WhereClause,
    pub joins: Joins,
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
            _phantom: PhantomData,
        }
    }

    /// Add a LIMIT to the query.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Limit::new(limit);
        self
    }

    /// Add an OFFSET.
    pub fn offset(mut self, offset: usize) -> Self {
        self.limit = self.limit.offset(offset);
        self
    }

    /// Add an ORDER BY clause.
    pub fn order_by(mut self, order_by: OrderBy) -> Self {
        self.order_by = order_by;
        self
    }

    fn filter(
        mut self,
        column: impl ToColumn,
        value: impl ToValue,
        join_op: JoinOp,
        not: bool,
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

        match join_op {
            JoinOp::And => self.where_clause.concat(filter),
            JoinOp::Or => self.where_clause.or(filter),
        };

        self
    }

    pub fn or(self, _query: Self) -> Self {
        todo!()
        // let other_filter = query.where_clause.filter();
        // let other_placeholders = query.placeholders;
        // self.where_clause.or(query.where_clause.filter());
        // self
    }

    pub fn filter_and(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::And, false);
        self
    }

    pub fn filter_or(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::Or, false);
        self
    }

    pub fn filter_not(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::And, true);
        self
    }

    pub fn filter_or_not(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        self = self.filter(column, value, JoinOp::Or, true);
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
}

impl<T: FromRow> ToSql for Select<T> {
    fn to_sql(&self) -> String {
        format!(
            r#"SELECT {} FROM "{}"{}{}{}{}"#,
            self.columns.to_sql(),
            self.table_name.escape(),
            self.joins.to_sql(),
            self.where_clause.to_sql(),
            self.order_by.to_sql(),
            self.limit.to_sql()
        )
    }
}
