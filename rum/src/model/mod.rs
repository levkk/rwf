use time::{OffsetDateTime, PrimitiveDateTime};

pub trait ToSql {
    fn to_sql(&self) -> String;
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
    Equal((Column, Column)),
    In((Column, Values)), // LessThan(Column),
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
            And(comparison) => format!("AND ({})", comparison.to_sql()),
            Or(comparison) => format!("OR ({})", comparison.to_sql()),
        }
    }
}

impl ToSql for Comparison {
    fn to_sql(&self) -> String {
        use Comparison::*;

        match self {
            Equal((a, b)) => format!(r#"{} = {}"#, a.to_sql(), b.to_sql()),
            In((column, values)) => format!(r#"{} IN ({})"#, column.to_sql(), values.to_sql()),
        }
    }
}

#[derive(Debug, Default)]
pub struct Where {
    comparisons: Vec<ComparisonOp>,
}

impl ToSql for Where {
    fn to_sql(&self) -> String {
        if self.comparisons.is_empty() {
            "".into()
        } else {
            format!(" WHERE {}", self.comparisons
                .iter()
                .map(|comparison| comparison.to_sql())
                .collect::<Vec<_>>()
                .join(" ")
            )
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
        limit: Limit,
    }
}

impl ToSql for  Query {
    fn to_sql(&self) -> String {
        use Query::*;

        match self {
            Select { table_name, columns, where_, limit } => format!(
                r#"SELECT {} FROM "{}"{}{}"#,
                columns.to_sql(),
                table_name,
                where_.to_sql(),
                limit.to_sql()
            )
        }
    }
}

pub trait Model {
    fn table_name() -> String;

    fn take_one() -> Query {
        Query::Select {
            table_name: Self::table_name(),
            limit: Limit::new(1),
            columns: Columns::default(),
            where_: Where::default(),
        }
    }

    fn take_many(n: usize) -> Query {
        Query::Select {
            table_name: Self::table_name(),
            limit: Limit::new(n),
            columns: Columns::default(),
            where_: Where::default(),
        }
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
}
