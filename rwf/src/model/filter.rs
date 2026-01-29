//! Implements the `WHERE` clause for `SELECT`, `UPDATE`, and `DELETE` statements.

use std::ops::{Deref, Not};

use super::{Column, ToSql, ToValue, Value};

/// The WHERE clause of a SQL query.
#[derive(Debug, Default, Clone, crate::prelude::Deserialize, crate::prelude::Serialize)]
pub struct WhereClause {
    filter: Filter,
}

#[derive(Debug, Clone, crate::prelude::Deserialize, crate::prelude::Serialize)]
pub(super) enum Comparison {
    /// x = 1
    Equal((Column, Value)),
    /// x IN (1, 2, 3)
    In((Column, Value)),
    /// (x = 1 AND y = 2)
    Filter(Filter),
    /// x > 1
    GreaterThan((Column, Value)),
    /// x < 1
    LesserThan((Column, Value)),
    /// x >= 1
    GreaterEqualThan((Column, Value)),
    /// x <= 1
    LesserEqualThan((Column, Value)),
    /// x LIKE '%hello%'
    Contains((Column, Value)),
    /// x LIKE 'hello%'
    StartsWith((Column, Value)),
    /// x LIKE '%hello'
    EndsWith((Column, Value)),
    /// Negates the inner operation.
    /// x = y => x <> y
    Negation(Box<Self>),
}
impl Not for Comparison {
    type Output = Self;
    fn not(self) -> Self::Output {
        use Comparison::*;
        match self {
            Negation(comparison) => (*comparison).into(),
            GreaterThan((col, val)) => LesserEqualThan((col, val)),
            GreaterEqualThan((col, val)) => LesserThan((col, val)),
            LesserThan((col, val)) => GreaterEqualThan((col, val)),
            LesserEqualThan((col, val)) => GreaterThan((col, val)),
            comp => Negation(Box::new(comp)),
        }
    }
}

impl Comparison {
    pub(super) fn new(op: super::select::Op, column: Column, value: Value) -> Self {
        use super::select::Op;
        match op {
            Op::Equals => match value {
                Value::Record(val) => Self::In((column, *val)),
                val => Self::Equal((column, val)),
            },
            Op::LesserThan => Self::LesserThan((column, value)),
            Op::GreaterThan => Self::GreaterThan((column, value)),
            Op::GreaterEqualThan => Self::GreaterEqualThan((column, value)),
            Op::LesserEqualThan => Self::LesserEqualThan((column, value)),
            Op::StartsWith => Self::StartsWith((column, value)),
            Op::EndsWith => Self::EndsWith((column, value)),
            Op::Contains => Self::Contains((column, value)),
            Op::Negation(inner) => !Self::new(*inner, column, value),
        }
    }
    fn placeholder(&self) -> bool {
        use Comparison::*;

        match self {
            Equal((_, v)) => v.placeholder(),
            In((_, v)) => v.placeholder(),
            GreaterThan((_, v)) => v.placeholder(),
            LesserThan((_, v)) => v.placeholder(),
            GreaterEqualThan((_, v)) => v.placeholder(),
            LesserEqualThan((_, v)) => v.placeholder(),
            Contains((_, v)) => v.placeholder(),
            StartsWith((_, v)) => v.placeholder(),
            EndsWith((_, v)) => v.placeholder(),
            Negation(v) => v.placeholder(),
            _ => false,
        }
    }
    fn add_offset(&mut self, offset: i32) {
        use Comparison::*;
        if let Value::Placeholder(val) = match self {
            Equal((_, v)) => v,
            In((_, v)) => v,
            GreaterThan((_, v)) => v,
            GreaterEqualThan((_, v)) => v,
            LesserThan((_, v)) => v,
            LesserEqualThan((_, v)) => v,
            Contains((_, v)) => v,
            StartsWith((_, v)) => v,
            EndsWith((_, v)) => v,
            Negation(v) => return v.add_offset(offset),
            _ => return,
        } {
            *val += offset;
        }
    }
}

impl ToSql for Comparison {
    fn to_sql(&self) -> String {
        use Comparison::*;

        match self {
            Equal((a, b)) => {
                if b.is_null() {
                    format!("{} IS NULL", a.to_sql())
                } else {
                    format!("{} = {}", a.to_sql(), b.to_sql())
                }
            }
            In((column, value)) => format!("{} = ANY({})", column.to_sql(), value.to_sql()),
            Filter(filter) => format!("({})", filter.to_sql()),
            GreaterThan((column, value)) => format!("{} > {}", column.to_sql(), value.to_sql()),
            LesserThan((column, value)) => format!("{} < {}", column.to_sql(), value.to_sql()),
            GreaterEqualThan((column, value)) => {
                format!("{} >= {}", column.to_sql(), value.to_sql())
            }
            LesserEqualThan((column, value)) => {
                format!("{} <= {}", column.to_sql(), value.to_sql())
            }
            Contains((column, value)) => {
                format!("{} LIKE '%' || {} || '%'", column.to_sql(), value.to_sql())
            }
            StartsWith((column, value)) => {
                format!("{} LIKE {} || '%'", column.to_sql(), value.to_sql())
            }
            EndsWith((column, value)) => {
                format!("{} LIKE '%' || {}", column.to_sql(), value.to_sql())
            }
            Negation(inner) => match inner.deref() {
                Equal((a, b)) => {
                    if b.is_null() {
                        format!("{} IS NOT NULL", a.to_sql())
                    } else {
                        format!("{} <> {}", a.to_sql(), b.to_sql())
                    }
                }
                In((column, value)) => format!("{} <> ANY({})", column.to_sql(), value.to_sql()),
                Filter(filter) => format!("NOT ({})", filter.to_sql()),
                Contains((column, value)) => {
                    format!(
                        "{} NOT LIKE '%' || {} || '%'",
                        column.to_sql(),
                        value.to_sql()
                    )
                }
                StartsWith((column, value)) => {
                    format!("{} NOT LIKE {} || '%'", column.to_sql(), value.to_sql())
                }
                EndsWith((column, value)) => {
                    format!("{} NOT LIKE '%' || {}", column.to_sql(), value.to_sql())
                }
                comp => comp.to_sql(),
            },
        }
    }
}

impl WhereClause {
    /// Add predicates to the WHERE clause using OR operator.
    pub fn or(&mut self, filter: Filter) {
        self.filter = self.filter.or(filter);
    }

    /// Add predicates to the WHERE clause using AND operator.
    pub fn and(&mut self, filter: Filter) {
        self.filter = self.filter.and(filter);
    }

    /// Add a single predicate to the WHERE clause, using the AND operator.
    pub fn add(&mut self, column: Column, value: impl ToValue) {
        self.filter.add(column, value);
    }

    /// Add a > predicate.
    pub fn gt(&mut self, column: Column, value: impl ToValue) {
        self.filter.gt(column, value);
    }

    /// Append all predicates of the filter into the current WHERE clause, e.g.
    /// (x = 1) "concat" (y = 2 AND z = 3) becomes (x = 1 AND y = 2 AND z = 3).
    pub fn concat(&mut self, filter: Filter) {
        self.filter = self.filter.concat(filter);
    }

    /// Remove all predicates.
    pub fn clear(&mut self) {
        self.filter.clauses.clear();
    }

    /// Clone the current filter.
    pub fn filter(&self) -> Filter {
        self.filter.clone()
    }

    pub fn insert_columns(&self) -> (Vec<Column>, Vec<Value>) {
        self.filter.insert_columns()
    }

    pub fn placeholders(&self) -> usize {
        self.filter.placeholders()
    }
    pub fn add_offset(&mut self, offset: i32) {
        self.filter.add_offset(offset)
    }
}

impl ToSql for WhereClause {
    fn to_sql(&self) -> String {
        if self.filter.is_empty() {
            "".to_string()
        } else {
            format!(" WHERE {}", self.filter.to_sql())
        }
    }
}

/// Type of connecting operation between two filters.
#[derive(
    Debug, Clone, Default, PartialEq, Copy, crate::prelude::Deserialize, crate::prelude::Serialize,
)]
pub enum JoinOp {
    /// AND
    #[default]
    And,
    /// OR
    Or,
}

impl ToSql for JoinOp {
    fn to_sql(&self) -> String {
        use JoinOp::*;

        match self {
            And => "AND",
            Or => "OR",
        }
        .to_string()
    }
}

/// A filter to be applied using the WHERE clause.
///
/// A filter is composed of multiple predicates joined by an operator,
/// e.g. AND.
///
/// # Example
///
/// ```sql
/// WHERE x = 1 AND b = 2
/// ```
///
#[derive(Debug, Clone, Default, crate::prelude::Deserialize, crate::prelude::Serialize)]
pub struct Filter {
    clauses: Vec<Comparison>,
    op: JoinOp,
}

impl Filter {
    /// Merge a filter using the OR operator, e.g.
    /// (x = 1) OR (y = 2 AND z = 3).
    pub fn or(&self, filter: Filter) -> Self {
        self.join(JoinOp::Or, filter)
    }

    /// Merge a filter using the AND operator, e.g.
    /// (x = 1) AND (y = 2 AND z = 3).
    pub fn and(&self, filter: Filter) -> Self {
        self.join(JoinOp::And, filter)
    }

    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }

    pub(super) fn push(&mut self, clause: Comparison) {
        self.clauses.push(clause);
    }

    /// Add a predicate to the filter, using the AND operator.
    pub fn add(&mut self, column: Column, value: impl ToValue) {
        let value = value.to_value();
        match value {
            Value::Record(value) => {
                self.clauses.push(Comparison::In((column, *value)));
            }
            value => {
                self.clauses.push(Comparison::Equal((column, value)));
            }
        }
    }

    pub fn starts_with(&mut self, column: Column, value: impl ToValue) {
        self.clauses
            .push(Comparison::StartsWith((column, value.to_value())));
    }
    pub fn ends_with(&mut self, column: Column, value: impl ToValue) {
        self.clauses
            .push(Comparison::EndsWith((column, value.to_value())));
    }
    pub fn contains(&mut self, column: Column, value: impl ToValue) {
        self.clauses
            .push(Comparison::Contains((column, value.to_value())));
    }
    pub fn gt(&mut self, column: Column, value: impl ToValue) {
        self.clauses
            .push(Comparison::GreaterThan((column, value.to_value())));
    }

    pub fn gte(&mut self, column: Column, value: impl ToValue) {
        self.clauses
            .push(Comparison::GreaterEqualThan((column, value.to_value())));
    }

    pub fn lt(&mut self, column: Column, value: impl ToValue) {
        self.clauses
            .push(Comparison::LesserThan((column, value.to_value())));
    }

    pub fn lte(&mut self, column: Column, value: impl ToValue) {
        self.clauses
            .push(Comparison::LesserEqualThan((column, value.to_value())));
    }

    pub fn negate_last(&mut self) -> () {
        if let Some(clause) = self.clauses.pop() {
            self.clauses.push(!clause)
        }
    }

    /// Append all predicates of the filter into the current filter.
    pub fn concat(&self, filter: Filter) -> Self {
        // Concatenating filters with different operations, e.g. AND and OR
        // will create incorrect queries.
        //
        // Use [`Self::join`] instead.
        assert_eq!(self.op, filter.op);

        let mut clauses = self.clauses.clone();
        clauses.extend(filter.clauses);
        Filter {
            clauses,
            op: self.op,
        }
    }

    pub fn placeholders(&self) -> usize {
        self.clauses
            .iter()
            .map(|op| match op {
                Comparison::Filter(filter) => filter.placeholders(),
                op => {
                    if op.placeholder() {
                        1
                    } else {
                        0
                    }
                }
            })
            .sum()
    }
    pub fn add_offset(&mut self, offset: i32) {
        self.clauses
            .iter_mut()
            .for_each(|clause| clause.add_offset(offset));
    }

    pub fn insert_columns(&self) -> (Vec<Column>, Vec<Value>) {
        let (mut columns, mut values) = (vec![], vec![]);
        for op in &self.clauses {
            match op {
                Comparison::Equal((column, value)) => {
                    columns.push(column.clone());
                    values.push(value.clone());
                }
                Comparison::Filter(filter) => {
                    let (c, v) = filter.insert_columns();
                    columns.extend(c);
                    values.extend(v);
                }
                _ => (),
            }
        }

        (columns, values)
    }

    fn join(&self, op: JoinOp, filter: Filter) -> Self {
        if self.is_empty() {
            filter
        } else {
            Filter {
                clauses: vec![Comparison::Filter(self.clone()), Comparison::Filter(filter)],
                op,
            }
        }
    }
}

impl ToSql for Filter {
    fn to_sql(&self) -> String {
        self.clauses
            .iter()
            .map(|s| s.to_sql().to_string())
            .collect::<Vec<_>>()
            .join(&format!(" {} ", self.op.to_sql()))
    }
}

#[cfg(test)]
mod test {
    use super::super::{Column, Value};
    use super::*;

    #[test]
    fn test_filter() {
        let filter = Filter {
            clauses: vec![
                Comparison::Equal((
                    Column::new("table_name", "column_a"),
                    Value::String("value".into()),
                )),
                !Comparison::Equal((Column::new("table_name", "column_b"), Value::Integer(42))),
                Comparison::Filter(Filter {
                    clauses: vec![
                        !Comparison::In((
                            Column::new("table_x", "column_y"),
                            Value::List(vec![Value::Integer(56), Value::Integer(67)]),
                        )),
                        Comparison::Equal((
                            Column::new("table_y", "column_x"),
                            Value::String("hello".into()),
                        )),
                    ],
                    op: JoinOp::Or,
                }),
            ],
            op: JoinOp::And,
        };

        let sql = filter.to_sql();
        assert_eq!(
            sql,
            r#""table_name"."column_a" = 'value' AND "table_name"."column_b" <> 42 AND ("table_x"."column_y" <> ANY({56, 67}) OR "table_y"."column_x" = 'hello')"#
        );
    }

    #[test]
    fn test_join() {
        let a = Filter {
            clauses: vec![
                Comparison::Equal((Column::new("table", "column_a"), Value::Integer(5))),
                !Comparison::Equal((Column::new("table", "column_a"), Value::Integer(125))),
            ],
            op: JoinOp::Or,
        };

        let b = Filter {
            clauses: vec![
                Comparison::Equal((Column::new("table", "column_b"), Value::Integer(42))),
                !Comparison::Equal((Column::new("table", "column_b"), Value::Integer(56))),
            ],
            op: JoinOp::And,
        };

        let or = a.clone().or(b.clone());
        let and = a.and(b);

        assert_eq!(
            and.to_sql(),
            r#"("table"."column_a" = 5 OR "table"."column_a" <> 125) AND ("table"."column_b" = 42 AND "table"."column_b" <> 56)"#
        );
        assert_eq!(
            or.to_sql(),
            r#"("table"."column_a" = 5 OR "table"."column_a" <> 125) OR ("table"."column_b" = 42 AND "table"."column_b" <> 56)"#
        );
    }
}
