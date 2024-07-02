use super::{Column, Comparison, ToSql, ToValue, Value};

#[derive(Debug, Default)]
pub struct WhereClause {
    filter: Filter,
}

impl WhereClause {
    pub fn or(&mut self, filter: Filter) {
        self.filter = self.filter.or(filter);
    }

    pub fn and(&mut self, filter: Filter) {
        self.filter = self.filter.and(filter);
    }

    pub fn add(&mut self, column: Column, value: impl ToValue) {
        self.filter.add(column, value);
    }

    pub fn concat(&mut self, filter: Filter) {
        self.filter = self.filter.concat(filter);
    }

    pub fn clear(&mut self) {
        self.filter.clauses.clear();
    }

    pub fn filter(&self) -> Filter {
        self.filter.clone()
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

#[derive(Debug, Clone, Default, PartialEq, Copy)]
pub enum JoinOp {
    #[default]
    And,
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

#[derive(Debug, Clone, Default)]
pub struct Filter {
    clauses: Vec<Comparison>,
    op: JoinOp,
}

impl Filter {
    pub fn or(&self, filter: Filter) -> Self {
        self.join(JoinOp::Or, filter)
    }

    pub fn and(&self, filter: Filter) -> Self {
        self.join(JoinOp::And, filter)
    }

    pub fn is_empty(&self) -> bool {
        self.clauses.is_empty()
    }

    // pub fn new(table_name: &str, filters: &[(impl ToString, impl ToValue)]) -> Self {
    //     let mut filter = Filter::default();
    //     let table_name = select.table_name.clone();

    //     for (column, value) in filters.into_iter() {
    //         let column = Column::new(&table_name, &column.to_string().as_str());
    //         let value = value.to_value();

    //         let value = match value {
    //             Value::List(_) => {
    //                 let placeholder = select.placeholders_mut().add(&value);
    //                 Value::Record(Box::new(placeholder))
    //             }

    //             value => select.placeholders_mut().add(&value),
    //         };

    //         filter.add(column, value);
    //     }
    //     Self::default()
    // }

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

    pub fn concat(&self, filter: Filter) -> Self {
        assert_eq!(self.op, filter.op);
        let mut clauses = self.clauses.clone();
        clauses.extend(filter.clauses);
        Filter {
            clauses,
            op: self.op,
        }
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
            .map(|s| format!("{}", s.to_sql()))
            .collect::<Vec<_>>()
            .join(&format!(" {} ", &self.op.to_sql()))
    }
}

#[cfg(test)]
mod test {
    use super::super::{Column, Comparison, Value};
    use super::*;

    #[test]
    fn test_filter() {
        let filter = Filter {
            clauses: vec![
                Comparison::Equal((
                    Column::new("table_name", "column_a"),
                    Value::String("value".into()),
                )),
                Comparison::NotEqual((Column::new("table_name", "column_b"), Value::Integer(42))),
                Comparison::Filter(Filter {
                    clauses: vec![
                        Comparison::NotIn((
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
                Comparison::NotEqual((Column::new("table", "column_a"), Value::Integer(125))),
            ],
            op: JoinOp::Or,
        };

        let b = Filter {
            clauses: vec![
                Comparison::Equal((Column::new("table", "column_b"), Value::Integer(42))),
                Comparison::NotEqual((Column::new("table", "column_b"), Value::Integer(56))),
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
