use time::{OffsetDateTime, PrimitiveDateTime};

pub trait ToSql {
    fn to_sql(&self) -> String;
}

impl ToSql for i32 {
    fn to_sql(&self) -> String {
        self.to_string()
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    TimestampT(OffsetDateTime),
    Timestamp(PrimitiveDateTime),
}

impl ToSql for Value {
    fn to_sql(&self) -> String {
        use Value::*;

        match self {
            Value::String(string) => format!("'{}'", string),
            Integer(integer) => integer.to_string(),
            Float(float) => float.to_string(),
            _ => todo!(),
        }
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

#[derive(Debug)]
pub struct Column {
    table_name: String,
    column_name: String,
}

impl ToSql for Column {
    fn to_sql(&self) -> String {
        format!(r#""{}"."{}""#, self.table_name, self.column_name)
    }
}

impl Column {
    pub fn new(table_name: impl ToString, column_name: impl ToString) -> Self {
        Self {
            table_name: table_name.to_string(),
            column_name: column_name.to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Columns {
    columns: Vec<Column>,
}

impl ToSql for Columns {
    fn to_sql(&self) -> String {
        if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns
                .iter()
                .map(|column| column.to_sql())
                .collect::<Vec<_>>()
                .join(", ")
        }
    }
}

pub struct Join {
    table_name: String,
    on: (String, String),
}

#[derive(Debug)]
pub struct Values {
    values: Vec<Value>,
}

impl ToSql for Values {
    fn to_sql(&self) -> String {
        self.values
            .iter()
            .map(|value| value.to_sql())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Debug)]
pub enum Comparison {
    Equal((Column, i32)),
    In((Column, i32)), // LessThan(Column),
                       // GreaterThan(Column),
}

#[derive(Debug)]
pub enum ComparisonOp {
    And(Comparison),
    Or(Comparison),
}

impl ToSql for ComparisonOp {
    fn to_sql(&self) -> String {
        use ComparisonOp::*;

        match self {
            And(comparison) => format!("({})", comparison.to_sql()),
            Or(comparison) => format!("({})", comparison.to_sql()),
        }
    }
}

impl ToSql for Comparison {
    fn to_sql(&self) -> String {
        use Comparison::*;

        match self {
            Equal((a, b)) => format!(r#"{} = ${}"#, a.to_sql(), b.to_sql()),
            In((column, values)) => format!(r#"{} IN (${})"#, column.to_sql(), values.to_sql()),
        }
    }
}

#[derive(Debug)]
pub enum OrderColumn {
    Asc(Column),
    Desc(Column),
}

impl ToSql for OrderColumn {
    fn to_sql(&self) -> String {
        use OrderColumn::*;

        match self {
            Asc(column) => format!("{} ASC", column.to_sql()),
            Desc(column) => format!("{} DESC", column.to_sql()),
        }
    }
}

#[derive(Debug, Default)]
pub struct OrderBy {
    order_by: Vec<OrderColumn>,
}

impl ToSql for OrderBy {
    fn to_sql(&self) -> String {
        if self.order_by.is_empty() {
            "".to_string()
        } else {
            format!(
                " ORDER BY {}",
                self.order_by
                    .iter()
                    .map(|column| column.to_sql())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

#[derive(Debug, Default)]
pub struct Where {
    comparisons: Vec<ComparisonOp>,
    values: Vec<Value>,
}

impl ToSql for Where {
    fn to_sql(&self) -> String {
        if self.comparisons.is_empty() {
            "".into()
        } else {
            let mut where_ = " WHERE ".to_string();

            for (idx, comparison) in self.comparisons.iter().enumerate() {
                if idx != 0 {
                    match comparison {
                        ComparisonOp::And(_) => where_.push_str(" AND "),
                        ComparisonOp::Or(_) => where_.push_str(" OR "),
                    }
                }

                where_.push_str(comparison.to_sql().as_str());
            }

            where_
        }
    }
}

#[derive(Debug, Default)]
pub struct Limit {
    limit: Option<i64>,
    offset: Option<i64>,
}

impl ToSql for Limit {
    fn to_sql(&self) -> String {
        let mut limit = String::new();
        if let Some(ref rows) = self.limit {
            limit += format!(" LIMIT {}", rows).as_str();
        }

        if let Some(ref offset) = self.offset {
            limit += format!(" OFFSET {}", offset).as_str();
        }

        limit
    }
}

impl Limit {
    fn new(n: usize) -> Self {
        Self {
            limit: Some(n as i64),
            offset: None,
        }
    }
}

#[derive(Debug)]
pub enum Query {
    Select {
        table_name: String,
        columns: Columns,
        where_: Where,
        order_by: OrderBy,
        limit: Limit,
    },
}

impl ToSql for Query {
    fn to_sql(&self) -> String {
        use Query::*;

        match self {
            Select {
                table_name,
                columns,
                where_,
                order_by,
                limit,
            } => format!(
                r#"SELECT {} FROM "{}"{}{}{}"#,
                columns.to_sql(),
                table_name,
                where_.to_sql(),
                order_by.to_sql(),
                limit.to_sql()
            ),
        }
    }
}

impl Query {
    fn select(table_name: impl ToString) -> Self {
        Query::Select {
            table_name: table_name.to_string(),
            limit: Limit::default(),
            columns: Columns::default(),
            where_: Where::default(),
            order_by: OrderBy::default(),
        }
    }

    fn take_one(mut self) -> Self {
        use Query::*;

        match self {
            Select {
                table_name,
                limit: _,
                columns,
                where_,
                order_by,
            } => Select {
                table_name,
                limit: Limit::new(1),
                columns,
                where_,
                order_by,
            },

            _ => unreachable!(),
        }
    }

    fn take_many(mut self, n: usize) -> Self {
        use Query::*;

        match self {
            Select {
                table_name,
                limit: _,
                columns,
                where_,
                order_by,
            } => Select {
                table_name,
                limit: Limit::new(n),
                columns,
                where_,
                order_by,
            },

            _ => unreachable!(),
        }
    }

    fn first_one(self) -> Query {
        use Query::*;

        match self {
            Select {
                table_name,
                limit: _,
                columns,
                where_,
                order_by: _,
            } => Select {
                limit: Limit::new(1),
                columns,
                where_,
                order_by: OrderBy {
                    order_by: vec![OrderColumn::Asc(Column::new(table_name.as_str(), "id"))],
                },
                table_name,
            },

            _ => unreachable!(),
        }
    }

    fn first_many(self, n: usize) -> Query {
        use Query::*;

        match self {
            Select {
                table_name,
                limit: _,
                columns,
                where_,
                order_by: _,
            } => Select {
                limit: Limit::new(n),
                columns,
                where_,
                order_by: OrderBy {
                    order_by: vec![OrderColumn::Asc(Column::new(table_name.as_str(), "id"))],
                },
                table_name,
            },

            _ => unreachable!(),
        }
    }

    pub fn filter(self, filter: &[(String, Value)]) -> Query {
        use Query::*;

        match self {
            Select {
                table_name,
                limit,
                columns,
                mut where_,
                order_by,
            } => {
                let start = where_.comparisons.len();
                for (idx, column) in filter.into_iter().enumerate() {
                    where_
                        .comparisons
                        .push(ComparisonOp::And(Comparison::Equal((
                            Column::new(&table_name, &column.0),
                            (idx + start) as i32 + 1,
                        ))));

                    where_.values.push(column.1.clone());
                }
                Select {
                    table_name,
                    limit,
                    columns,
                    where_,
                    order_by,
                }
            }

            _ => unreachable!(),
        }
    }
}

pub trait Model {
    fn table_name() -> String;

    fn primary_key() -> String {
        "id".to_string()
    }

    fn take_one() -> Query {
        Query::select(Self::table_name()).take_one()
    }

    fn take_many(n: usize) -> Query {
        Query::select(Self::table_name()).take_many(n)
    }

    fn first_one() -> Query {
        Query::select(Self::table_name()).first_one()
    }

    fn first_many(n: usize) -> Query {
        Query::select(Self::table_name()).first_many(n)
    }

    fn all() -> Query {
        Query::select(Self::table_name())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_take_one() {
        struct Users;
        impl Model for Users {
            fn table_name() -> String {
                "users".into()
            }
        }

        let query = Users::take_one().to_sql();

        assert_eq!(query, r#"SELECT * FROM "users" LIMIT 1"#);
    }

    #[test]
    fn test_take_many() {
        struct Users;
        impl Model for Users {
            fn table_name() -> String {
                "users".into()
            }
        }

        let query = Users::take_many(25).to_sql();

        assert_eq!(query, r#"SELECT * FROM "users" LIMIT 25"#);
    }

    #[test]
    fn test_first_one() {
        struct Users;
        impl Model for Users {
            fn table_name() -> String {
                "users".into()
            }
        }

        let query = Users::first_one().to_sql();

        assert_eq!(
            query,
            r#"SELECT * FROM "users" ORDER BY "users"."id" ASC LIMIT 1"#
        );
    }

    #[test]
    fn test_first_many() {
        struct Users;
        impl Model for Users {
            fn table_name() -> String {
                "users".into()
            }
        }

        let query = Users::first_many(25).to_sql();

        assert_eq!(
            query,
            r#"SELECT * FROM "users" ORDER BY "users"."id" ASC LIMIT 25"#
        );
    }

    #[test]
    fn test_all() {
        struct Users;
        impl Model for Users {
            fn table_name() -> String {
                "users".into()
            }
        }

        let query = Users::all().to_sql();

        assert_eq!(query, r#"SELECT * FROM "users""#);
    }

    #[test]
    fn test_filter() {
        struct Users;
        impl Model for Users {
            fn table_name() -> String {
                "users".into()
            }
        }

        let query = Users::all()
            .filter(&[
                ("email".into(), "test@test.com".into()),
                ("password".into(), "not_encrypted".into()),
            ])
            .filter(&[("id".into(), 5.into())]);

        assert_eq!(
            query.to_sql(),
            r#"SELECT * FROM "users" WHERE ("users"."email" = $1) AND ("users"."password" = $2) AND ("users"."id" = $3)"#
        );
    }
}
