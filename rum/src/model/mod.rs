pub mod column;
pub mod error;
pub mod escape;
pub mod filter;
pub mod limit;
pub mod order_by;
pub mod placeholders;
pub mod pool;
pub mod row;
pub mod select;
pub mod value;

pub use column::{Column, Columns};
pub use error::Error;
pub use escape::Escape;
pub use filter::{Filter, WhereClause};
pub use limit::Limit;
pub use order_by::{OrderBy, OrderColumn, ToOrderBy};
pub use placeholders::Placeholders;
pub use pool::Pool;
pub use row::Row;
pub use select::{Select, ToFilterable};
pub use value::{ToValue, Value, Values};

pub trait ToSql {
    fn to_sql(&self) -> String;
}

#[allow(dead_code)]
struct Join {
    table_name: String,
    on: (String, String),
}

#[derive(Debug)]
pub enum Query {
    Select(Select),
    Update,
    Raw(String),
}

impl ToSql for Query {
    fn to_sql(&self) -> String {
        use Query::*;

        match self {
            Select(select) => select.to_sql(),
            Raw(query) => query.clone(),
            Update => todo!(),
        }
    }
}

impl Query {
    pub fn select(table_name: impl ToString) -> Self {
        Query::Select(Select {
            table_name: table_name.to_string(),
            primary_key: "id".to_string(),
            ..Default::default()
        })
    }

    pub fn take_one(self) -> Self {
        use Query::*;

        match self {
            Select(select) => Select(select.limit(1)),
            _ => unreachable!(),
        }
    }

    pub fn take_many(self, n: usize) -> Self {
        use Query::*;

        match self {
            Select(select) => Select(select.limit(n)),
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
                let order_by = if select.order_by.is_empty() {
                    OrderBy::asc(Column::new(table_name.as_str(), &select.primary_key))
                } else {
                    select.order_by.clone()
                };

                Select(select.limit(n).order_by(order_by))
            }

            _ => unreachable!(),
        }
    }

    pub fn filter(self, filters: &[(impl ToString, impl ToValue)]) -> Query {
        use Query::*;

        match self {
            Select(select) => Select(select.filter_and(filters)),
            _ => self,
        }
    }

    pub fn or(self, filters: &[(impl ToString, impl ToValue)]) -> Query {
        use Query::*;

        match self {
            Select(select) => Select(select.filter_or(filters)),
            _ => self,
        }
    }

    pub fn not(self, filters: &[(impl ToString, impl ToValue)]) -> Query {
        use Query::*;

        match self {
            Select(select) => Select(select.filter_not(filters)),
            _ => self,
        }
    }

    pub fn or_not(self, filters: &[(impl ToString, impl ToValue)]) -> Query {
        use Query::*;

        match self {
            Select(select) => Select(select.filter_or_not(filters)),
            _ => self,
        }
    }

    pub fn find_by(mut self, column: impl ToString, value: Value) -> Query {
        use Query::*;

        if let Select(select::Select {
            ref mut where_clause,
            ..
        }) = self
        {
            where_clause.clear();
        }

        self.filter(&[(column.to_string(), value)])
    }

    pub fn limit(self, limit: usize) -> Query {
        self.take_many(limit)
    }

    pub fn offset(self, offset: usize) -> Query {
        if let Query::Select(mut select) = self {
            Query::Select(select.offset(offset))
        } else {
            self
        }
    }

    pub fn order(self, order: impl ToOrderBy) -> Query {
        if let Query::Select(mut select) = self {
            select.order_by = select.order_by + order.to_order_by();
            Query::Select(select)
        } else {
            self
        }
    }

    async fn execute_internal(self, client: &tokio_postgres::Client) -> Result<Vec<Row>, Error> {
        let query = self.to_sql();

        let rows = match self {
            Query::Select(select) => {
                let values = select.placeholders.values();
                match client.query(&query, &values).await {
                    Ok(rows) => rows,
                    Err(err) => {
                        return Err(Error::QueryError(
                            query,
                            err.as_db_error().expect("db error").message().to_string(),
                        ))
                    }
                }
            }

            Query::Raw(query) => client.query(&query, &[]).await?,

            _ => vec![],
        };

        Ok(rows.into_iter().map(|row| Row::new(row)).collect())
    }

    /// Execute a query and return an optional result.
    pub async fn execute(self, pool: &Pool) -> Result<Vec<Row>, Error> {
        let conn = pool.get().await?;
        self.execute_internal(&conn).await
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

    fn filter(filters: &[(impl ToString, impl ToValue)]) -> Query {
        Query::select(Self::table_name()).filter(filters)
    }

    fn find_by(column: impl ToString, value: impl ToValue) -> Query {
        Query::select(Self::table_name())
            .find_by(column, value.to_value())
            .take_one()
    }

    fn find_by_sql(query: impl ToString) -> Query {
        Query::Raw(query.to_string())
    }

    fn order(order: impl ToOrderBy) -> Query {
        Self::all().order(order)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio_postgres::row::Row;
    use tokio_postgres::NoTls;

    #[allow(dead_code)]
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
            r#"SELECT * FROM "users" WHERE "users"."email" = $1 AND "users"."password" = $2 AND "users"."id" = $3 AND "users"."id" = ANY($4)"#
        );
    }

    #[test]
    fn test_find_by() {
        let query = User::find_by("email", "test@test.com");
        assert_eq!(
            query.to_sql(),
            r#"SELECT * FROM "users" WHERE "users"."email" = $1 LIMIT 1"#
        );
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

        let rows = User::order(("email", "ASC NULLS LAST"))
            .first_one()
            .execute_internal(&client)
            .await;

        assert_eq!(rows.expect("result").len(), 1);

        client.query("ROLLBACK", &[]).await.expect("rollback");
    }

    #[test]
    fn test_or() {
        let query = User::all()
            .filter(&[("email", "test@test.com")])
            .filter(&[("password", "not_encrypted")])
            .or(&[("email", "another@test.com")]);

        assert_eq!(
            query.to_sql(),
            r#"SELECT * FROM "users" WHERE ("users"."email" = $1 AND "users"."password" = $2) OR ("users"."email" = $3)"#
        );

        let query = User::all()
            .not(&[("email", "test@test.com")])
            .or_not(&[("email", "another@test.com")]);

        assert_eq!(
            query.to_sql(),
            r#"SELECT * FROM "users" WHERE ("users"."email" <> $1) OR ("users"."email" <> $2)"#
        );
    }
}
