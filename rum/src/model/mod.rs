use bytes::BytesMut;
use time::{OffsetDateTime, PrimitiveDateTime};
use tokio_postgres::{
    types::{to_sql_checked, Format, IsNull, Type},
    Client,
};

pub mod select;

pub use select::Select;

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
    Tuple(Vec<Value>),
}

pub trait ToValue {
    fn to_value(&self) -> Value;
}

impl ToValue for &str {
    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl ToValue for i64 {
    fn to_value(&self) -> Value {
        Value::Integer(*self)
    }
}

impl ToValue for Value {
    fn to_value(&self) -> Value {
        self.clone()
    }
}

impl ToValue for &[&str] {
    fn to_value(&self) -> Value {
        Value::Tuple(self.iter().map(|v| v.to_value()).collect::<Vec<_>>())
    }
}

impl ToValue for &[i64] {
    fn to_value(&self) -> Value {
        Value::Tuple(self.iter().map(|v| v.to_value()).collect::<Vec<_>>())
    }
}

pub trait Escape {
    fn escape(&self) -> String;
}

impl Escape for Value {
    fn escape(&self) -> String {
        use Value::*;

        match self {
            String(string) => string.escape(),
            Integer(integer) => format!("'{}'", integer),
            Float(float) => format!("'{}'", float),
            Tuple(values) => format!(
                "({})",
                values
                    .into_iter()
                    .map(|value| value.escape())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            _ => todo!(),
        }
    }
}

impl tokio_postgres::types::ToSql for Value {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        match self {
            Value::String(string) => string.to_sql(ty, out),
            Value::Integer(integer) => integer.to_sql(ty, out),
            Value::Float(float) => float.to_sql(ty, out),
            // Value::TimestampT(timestampt) => timestampt.to_sql(ty, out),
            _ => todo!(),
        }
    }

    fn accepts(ty: &Type) -> bool {
        todo!()
    }

    to_sql_checked!();
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
        format!(
            r#""{}"."{}""#,
            self.table_name.escape(),
            self.column_name.escape()
        )
    }
}

impl Escape for String {
    fn escape(&self) -> String {
        self.replace("\"", "\"\"").replace("'", "''")
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
    Select(Select),
    Update,
}

impl ToSql for Query {
    fn to_sql(&self) -> String {
        use Query::*;

        match self {
            Select(select::Select {
                table_name,
                columns,
                where_,
                order_by,
                limit,
            }) => format!(
                r#"SELECT {} FROM "{}"{}{}{}"#,
                columns.to_sql(),
                table_name,
                where_.to_sql(),
                order_by.to_sql(),
                limit.to_sql()
            ),

            Update => todo!(),
        }
    }
}

impl Query {
    pub fn select(table_name: impl ToString) -> Self {
        Query::Select(Select {
            table_name: table_name.to_string(),
            limit: Limit::default(),
            columns: Columns::default(),
            where_: Where::default(),
            order_by: OrderBy::default(),
        })
    }

    pub fn take_one(mut self) -> Self {
        use Query::*;

        match self {
            Select(mut select) => Select(select.limit(Limit::new(1))),
            _ => unreachable!(),
        }
    }

    pub fn take_many(mut self, n: usize) -> Self {
        use Query::*;

        match self {
            Select(mut select) => Select(select.limit(Limit::new(n))),
            _ => unreachable!(),
        }
    }

    pub fn first_one(self) -> Query {
        use Query::*;

        match self {
            Select(_) => self.first_many(1),
            _ => unreachable!(),
        }
    }

    pub fn first_many(self, n: usize) -> Query {
        use Query::*;

        match self {
            Select(mut select) => {
                let table_name = select.table_name.clone();
                Select(select.limit(Limit::new(n)).order_by(OrderBy {
                    order_by: vec![OrderColumn::Asc(Column::new(table_name.as_str(), "id"))],
                }))
            }

            _ => unreachable!(),
        }
    }

    pub fn filter(self, filter: &[(impl ToString, impl ToValue)]) -> Query {
        use Query::*;

        match self {
            Select(mut select) => {
                let start = select.where_mut().comparisons.len();
                let table_name = select.table_name.clone();
                for (idx, column) in filter.into_iter().enumerate() {
                    select
                        .where_mut()
                        .comparisons
                        .push(ComparisonOp::And(Comparison::Equal((
                            Column::new(&table_name, &column.0.to_string()),
                            (idx + start) as i32 + 1,
                        ))));

                    select.where_mut().values.push(column.1.to_value().clone());
                }

                Select(select)
            }

            _ => unreachable!(),
        }
    }

    pub fn find_by(mut self, column: impl ToString, value: Value) -> Query {
        use Query::*;

        if let Select(select::Select { ref mut where_, .. }) = self {
            where_.comparisons.clear();
            where_.values.clear();
        }

        self.filter(&[(column.to_string(), value)])
    }

    pub fn limit(self, limit: usize) -> Query {
        self.take_many(limit)
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

    fn find_by(column: impl ToString, value: impl ToValue) -> Query {
        Query::select(Self::table_name())
            .find_by(column, value.to_value())
            .take_one()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio_postgres::row::Row;

    struct User {
        id: i64,
        email: String,
        password: String,
    }

    impl TryFrom<Row> for User {
        type Error = tokio_postgres::Error;

        fn try_from(row: Row) -> Result<User, Self::Error> {
            let id: i64 = row.try_get::<_, i64>("id")?;
            let email: String = row.try_get::<_, String>("email")?;
            let password: String = row.try_get::<_, String>("password")?;

            Ok(User {
                id,
                email,
                password,
            })
        }
    }

    impl Model for User {
        fn table_name() -> String {
            "users".into()
        }
    }

    #[test]
    fn test_take_one() {
        let query = User::take_one().to_sql();

        assert_eq!(query, r#"SELECT * FROM "users" LIMIT 1"#);
    }

    #[test]
    fn test_take_many() {
        struct User;
        impl Model for User {
            fn table_name() -> String {
                "users".into()
            }
        }

        let query = User::take_many(25).to_sql();

        assert_eq!(query, r#"SELECT * FROM "users" LIMIT 25"#);
    }

    #[test]
    fn test_first_one() {
        let query = User::first_one().to_sql();

        assert_eq!(
            query,
            r#"SELECT * FROM "users" ORDER BY "users"."id" ASC LIMIT 1"#
        );
    }

    #[test]
    fn test_first_many() {
        let query = User::first_many(25).to_sql();

        assert_eq!(
            query,
            r#"SELECT * FROM "users" ORDER BY "users"."id" ASC LIMIT 25"#
        );
    }

    #[test]
    fn test_all() {
        let query = User::all().to_sql();

        assert_eq!(query, r#"SELECT * FROM "users""#);
    }

    #[test]
    fn test_filter() {
        let query = User::all()
            .filter(&vec![
                ("email", "test@test.com"),
                ("password", "not_encrypted"),
            ])
            .filter(&[("id", 5)]);

        assert_eq!(
            query.to_sql(),
            r#"SELECT * FROM "users" WHERE ("users"."email" = $1) AND ("users"."password" = $2) AND ("users"."id" = $3)"#
        );
    }

    #[test]
    fn test_find_by() {
        let query = User::find_by("email", "test@test.com");
    }
}
