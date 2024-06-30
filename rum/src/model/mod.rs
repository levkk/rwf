pub mod column;
pub mod error;
pub mod escape;
pub mod filter;
pub mod limit;
pub mod order_by;
pub mod select;
pub mod value;

pub use column::Column;
pub use error::Error;
pub use escape::Escape;
pub use limit::Limit;
pub use select::Select;
pub use value::{ToValue, Value, Values};

pub trait ToSql {
    fn to_sql(&self) -> String;
}

impl ToSql for i32 {
    fn to_sql(&self) -> String {
        self.to_string()
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

#[allow(dead_code)]
struct Join {
    table_name: String,
    on: (String, String),
}

#[derive(Debug)]
pub enum Comparison {
    Equal((Column, Value)),
    In((Column, Value)),
    NotIn((Column, Value)),
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
            Equal((a, b)) => format!(r#"{} = {}"#, a.to_sql(), b.to_sql()),
            In((column, value)) => format!(r#"{} = ANY({})"#, column.to_sql(), value.to_sql()),
            NotIn((column, value)) => format!(r#"{} <> ANY({})"#, column.to_sql(), value.to_sql()),
        }
    }
}

#[derive(Debug)]
pub enum OrderColumn {
    Asc(Column),
    Desc(Column),
    Raw(String),
}

impl ToSql for OrderColumn {
    fn to_sql(&self) -> String {
        use OrderColumn::*;

        match self {
            Asc(column) => format!("{} ASC", column.to_sql()),
            Desc(column) => format!("{} DESC", column.to_sql()),
            Raw(raw) => raw.clone(),
        }
    }
}

pub trait ToOrderBy {
    fn to_order_by(&self) -> OrderBy;
}

impl ToOrderBy for &str {
    fn to_order_by(&self) -> OrderBy {
        OrderBy {
            order_by: vec![OrderColumn::Raw(self.to_string())],
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

impl Where {
    pub fn values(&self) -> Vec<&(dyn tokio_postgres::types::ToSql + Sync)> {
        self.values
            .iter()
            .map(|v| v as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect()
    }
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
                table_name.escape(),
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

    pub fn take_one(self) -> Self {
        use Query::*;

        match self {
            Select(select) => Select(select.limit(Limit::new(1))),
            _ => unreachable!(),
        }
    }

    pub fn take_many(self, n: usize) -> Self {
        use Query::*;

        match self {
            Select(select) => Select(select.limit(Limit::new(n))),
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
            Select(select) => {
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
                    let value = column.1.to_value();
                    let column = Column::new(&table_name, &column.0.to_string());
                    let placeholder = Value::Placeholder((idx + start) as i32 + 1);
                    let comparison = match value {
                        Value::List(ref _value) => {
                            ComparisonOp::And(Comparison::In((column, placeholder)))
                        }
                        ref _value => ComparisonOp::And(Comparison::Equal((column, placeholder))),
                    };

                    select.where_mut().comparisons.push(comparison);

                    select.where_mut().values.push(value);
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

    pub fn order(self, _order: impl ToOrderBy) -> Query {
        todo!()
    }

    async fn execute_internal(
        self,
        client: &tokio_postgres::Client,
    ) -> Result<Vec<tokio_postgres::Row>, Error> {
        let query = self.to_sql();

        match self {
            Query::Select(select) => {
                let values = select.where_.values();
                Ok(client.query(&query, &values).await?)
            }

            _ => todo!(),
        }
    }

    pub async fn execute(
        self,
        client: &tokio_postgres::Client,
    ) -> Result<Vec<tokio_postgres::Row>, Error> {
        self.execute_internal(client).await
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
    use tokio_postgres::{Error, NoTls};

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
            .filter(&[("id", 5)])
            .filter(&[("id", [1_i64, 2, 3].as_slice())]);

        assert_eq!(
            query.to_sql(),
            r#"SELECT * FROM "users" WHERE ("users"."email" = $1) AND ("users"."password" = $2) AND ("users"."id" = $3) AND ("users"."id" = ANY($4))"#
        );
    }

    #[test]
    fn test_find_by() {
        let query = User::find_by("email", "test@test.com");
    }

    #[tokio::test]
    async fn test_execute() {
        let (client, connection) =
            tokio_postgres::connect("host=localhost user=lev password=lev", NoTls)
                .await
                .expect("connect");

        tokio::task::spawn(async move {
            let _ = connection.await;
        });

        client.query("BEGIN", &[]).await.expect("transaction");
        client
            .query(
                "CREATE TABLE users (id BIGINT, email VARCHAR, password VARCHAR);",
                &[],
            )
            .await
            .expect("table");
        client
            .query(
                "INSERT INTO users VALUES (1, 'test@test.com', 'not_encrypted')",
                &[],
            )
            .await
            .expect("insert");

        // let query = User::find_by("email", "test@test.com");
        // query.execute(&client).await;

        let rows = User::first_one().execute(&client).await;

        assert_eq!(rows.len(), 1);

        client.query("ROLLBACK", &[]).await.expect("rollback");
    }
}
