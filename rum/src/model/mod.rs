use colored::Colorize;
use once_cell::sync::OnceCell;
use std::time::Instant;
use tracing::info;

pub mod column;
pub mod error;
pub mod escape;
pub mod explain;
pub mod filter;
pub mod join;
pub mod limit;
pub mod macros;
pub mod order_by;
pub mod placeholders;
pub mod pool;
pub mod row;
pub mod select;
pub mod value;

pub use column::{Column, Columns, ToColumn};
pub use error::Error;
pub use escape::Escape;
pub use explain::Explain;
pub use filter::{Filter, WhereClause};
pub use join::{Association, AssociationType, Join, Joined, Joins};
pub use limit::Limit;
pub use macros::belongs_to;
pub use order_by::{OrderBy, OrderColumn, ToOrderBy};
pub use placeholders::Placeholders;
pub use pool::{IntoWrapper, Pool, Wrapper};
pub use row::Row;
pub use select::Select;
pub use value::{ToValue, Value, Values};

static POOL: OnceCell<Pool> = OnceCell::new();

/// Convert a PostgreSQL row to a Rust struct.
///
/// This trait needs to be implemented by all structs that are used
/// as models.
///
/// It's recommended to handle missing columns by using default values
/// instead of panicking. Missing columns could indicate the version
/// of the code is out of sync with the database, which could happen
/// because of a migration or manual intervention.
///
/// # Example
///
/// ```
/// use rum::model::FromRow;
///
/// #[derive(Clone)]
/// struct User {
///     id: i64,
///     email: String,
/// }
///
/// impl FromRow for User {
///     fn from_row(row: tokio_postgres::Row) -> Self {
///         let id: i64 = row.get("id");
///         let email: String = row.get("email");
///
///         User {
///             id,
///             email,
///         }
///     }
/// }   
/// ```
pub trait FromRow: Clone {
    fn from_row(row: tokio_postgres::Row) -> Self
    where
        Self: Sized;
}

/// Convert an entity to a valid SQL string.
///
/// This trait can be implemented for pretty much anything,
/// from a single table column to a multi-table join query.
/// It's the implementor's responsibility to make sure
/// all SQL is valid and user input is escaped to avoid SQL injection
/// attacks.
///
/// # Example
///
/// ```rust
/// use rum::model::{ToSql, Escape};
///
/// struct SelectUser {
///     email: String,
/// }
///
/// impl ToSql for SelectUser {
///     fn to_sql(&self) -> String {
///         format!("SELECT * FROM users WHERE email = '{}'", self.email.escape())
///     }
/// }
/// ```
///
pub trait ToSql {
    /// Convert `self` into a valid SQL entity.
    fn to_sql(&self) -> String;
}

#[derive(Debug)]
pub enum Query<T: FromRow + ?Sized> {
    Select(Select<T>),
    Update,
    Raw(String),
}

impl<T: FromRow> ToSql for Query<T> {
    fn to_sql(&self) -> String {
        use Query::*;

        match self {
            Select(select) => select.to_sql(),
            Raw(query) => query.clone(),
            Update => todo!(),
        }
    }
}

impl<T: Model> Query<T> {
    /// Start a SELECT query from the given relation.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the relation.
    ///
    /// # Example
    ///
    /// ```
    /// use rum::model::{Query, ToSql};
    ///
    /// let query = Query::select("users");
    /// assert_eq!(query.to_sql(), "SELECT * FROM \"users\"");
    /// ```
    pub fn select(table_name: impl ToString) -> Self {
        Query::Select(Select::new(
            table_name.to_string().as_str(),
            &T::primary_key(),
        ))
    }

    /// Create a query that selects one row from the relation.
    ///
    /// # Example
    ///
    /// ```
    /// use rum::model::{Query, ToSql};
    ///
    /// let query = Query::select("users").take_one();
    /// assert_eq!(query.to_sql(), "SELECT * FROM \"users\" LIMIT 1");
    /// ```
    pub fn take_one(self) -> Self {
        use Query::*;

        match self {
            Select(select) => Select(select.limit(1)),
            _ => unreachable!(),
        }
    }

    /// Create a query that selects _n_ rows from the relation.
    ///
    /// # Example
    ///
    /// ```
    /// use rum::model::{Query, ToSql, Explain};
    ///
    /// let query = Query::select("users").take_many(25).explain();
    /// assert_eq!(query.to_sql(), "EXPLAIN SELECT * FROM \"users\" LIMIT 25");
    /// ```
    pub fn take_many(self, n: usize) -> Self {
        use Query::*;

        match self {
            Select(select) => Select(select.limit(n)),
            _ => unreachable!(),
        }
    }

    pub fn first_one(self) -> Self {
        use Query::*;

        match self {
            Select(_) => self.first_many(1),
            _ => unreachable!(),
        }
    }

    pub fn first_many(self, n: usize) -> Self {
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

    pub fn filter(self, column: impl ToColumn, value: impl ToValue) -> Self {
        use Query::*;

        match self {
            Select(select) => Select(select.filter_and(column, value)),
            _ => self,
        }
    }

    // pub fn filter_gt(self, column: impl ToColumn, value: impl ToValue) -> Self {
    //     use Query::*;
    //     match self {
    //         Select(select) => Select(select.filter_gt(column, value)),
    //         _ => self,
    //     }
    // }

    pub fn or(self, _other: Query<T>) -> Self {
        // TODO:
        //
        // 1. merge the filters of both queries
        // 2. rewrite placeholders of the `other` query to start at id + 1
        todo!()
    }

    pub fn not(self, column: impl ToColumn, value: impl ToValue) -> Self {
        use Query::*;

        match self {
            Select(select) => Select(select.filter_not(column, value)),
            _ => self,
        }
    }

    pub fn or_not(self, column: impl ToColumn, value: impl ToValue) -> Self {
        use Query::*;

        match self {
            Select(select) => Select(select.filter_or_not(column, value)),
            _ => self,
        }
    }

    pub fn find_by(mut self, column: impl ToColumn, value: impl ToValue) -> Self {
        use Query::*;

        if let Select(select::Select {
            ref mut where_clause,
            ..
        }) = self
        {
            where_clause.clear();
        }

        self.filter(column, value).take_one()
    }

    pub fn limit(self, limit: usize) -> Self {
        self.take_many(limit)
    }

    pub fn offset(self, offset: usize) -> Self {
        if let Query::Select(select) = self {
            Query::Select(select.offset(offset))
        } else {
            self
        }
    }

    pub fn order(self, order: impl ToOrderBy) -> Self {
        if let Query::Select(mut select) = self {
            select.order_by = select.order_by + order.to_order_by();
            Query::Select(select)
        } else {
            self
        }
    }

    /// Join this relation with another relation directly related to it, either
    /// through a foreign key.
    ///
    /// Joined relation must implement [`Association`] for current relation.
    ///
    /// # Example
    ///
    /// ```
    /// use rum::model::{Association, Model};
    ///
    /// #[derive(Clone, Default)]
    /// struct User {}
    ///
    /// #[derive(Clone, Default)]
    /// struct Order {}
    ///
    /// impl Association<Order> for User {}
    /// ```
    pub fn join<F: Association<T>>(self) -> Self {
        match self {
            Query::Select(select) => Query::Select(select.join(F::construct_join())),
            _ => self,
        }
    }

    pub fn join_nested<F: Association<T>, G: Model>(self, joined: Joined<F, G>) -> Self {
        match self {
            Query::Select(select) => Query::Select(select.add_joins(joined.into())),
            _ => self,
        }
    }

    async fn execute_internal(
        &self,
        client: &tokio_postgres::Client,
    ) -> Result<Vec<tokio_postgres::Row>, Error> {
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

            Query::Raw(query) => client.query(query, &[]).await?,

            _ => vec![],
        };

        Ok(rows)
    }

    /// Execute the query and fetch the first row from the database.
    pub async fn fetch(self, conn: &tokio_postgres::Client) -> Result<T, Error> {
        match self.execute(conn).await?.first().cloned() {
            Some(row) => Ok(row),
            None => Err(Error::RecordNotFound),
        }
    }

    /// Execute the query and fetch all rows from the database.
    pub async fn fetch_all(self, conn: &tokio_postgres::Client) -> Result<Vec<T>, Error> {
        self.execute(conn).await
    }

    /// Get the query plan from Postgres.
    ///
    /// Take the actual query, prepend `EXPLAIN` and execute.
    pub async fn explain(self, conn: &tokio_postgres::Client) -> Result<Explain, Error> {
        let query = Query::<Explain>::Raw(format!("EXPLAIN {}", self.to_sql()));
        match query.execute_internal(conn).await?.pop() {
            Some(explain) => Ok(Explain::from_row(explain)),
            None => Err(Error::RecordNotFound),
        }
    }

    /// Execute a query and return an optional result.
    pub async fn execute(self, conn: &tokio_postgres::Client) -> Result<Vec<T>, Error> {
        let start = Instant::now();
        let result = self
            .execute_internal(conn)
            .await?
            .into_iter()
            .map(|row| T::from_row(row))
            .collect();
        let time = start.elapsed();

        info!(
            "{} {} ({:.2} ms) {}",
            "ORM".purple(),
            std::any::type_name::<T>().green(),
            time.as_secs_f64() * 1000.0,
            self.to_sql()
        );

        Ok(result)
    }
}

pub trait Model: FromRow {
    fn table_name() -> String {
        let name = std::any::type_name::<Self>()
            .split("::")
            .last()
            .expect("a struct to have a type name");
        pluralizer::pluralize(crate::snake_case(name).as_str(), 2, false)
    }

    fn column(name: &str) -> Column {
        Column::new(Self::table_name(), name)
    }

    fn foreign_key() -> String {
        format!(
            "{}_id",
            pluralizer::pluralize(&Self::table_name(), 1, false)
        )
    }

    fn relationships() -> Vec<Join> {
        vec![]
    }

    fn configure_pool(pool: Pool) -> Result<(), Error> {
        match POOL.set(pool) {
            Ok(()) => Ok(()),
            Err(_pool) => Err(Error::Unknown("pool already configured".into())),
        }
    }

    fn primary_key() -> String {
        "id".to_string()
    }

    // fn id(&self) -> i64 {
    //     0
    // }

    fn take_one() -> Query<Self> {
        Query::select(Self::table_name()).take_one()
    }

    fn take_many(n: usize) -> Query<Self> {
        Query::select(Self::table_name()).take_many(n)
    }

    fn first_one() -> Query<Self> {
        Query::select(Self::table_name()).first_one()
    }

    fn first_many(n: usize) -> Query<Self> {
        Query::select(Self::table_name()).first_many(n)
    }

    fn all() -> Query<Self> {
        Query::select(Self::table_name())
    }

    fn filter(column: impl ToColumn, value: impl ToValue) -> Query<Self> {
        Query::select(Self::table_name()).filter(column, value)
    }

    fn find_by(column: impl ToColumn, value: impl ToValue) -> Query<Self> {
        Query::select(Self::table_name())
            .find_by(column, value.to_value())
            .take_one()
    }

    fn find_by_sql(query: impl ToString) -> Query<Self> {
        Query::Raw(query.to_string())
    }

    fn order(order: impl ToOrderBy) -> Query<Self> {
        Self::all().order(order)
    }

    fn join<F: Association<Self>>() -> Joined<Self, F> {
        Joined::new(F::construct_join())
    }

    fn related<F: Association<Self>>(fks: &[i64]) -> Query<F> {
        F::all().filter(Self::foreign_key(), fks)
    }
}

#[cfg(test)]
mod test {
    use super::join::AssociationType;
    use super::*;
    use tokio_postgres::row::Row;
    use tokio_postgres::NoTls;

    #[derive(Debug, Clone, Default)]
    struct User {
        id: i64,
        email: String,
        password: String,
    }

    impl Model for User {
        fn table_name() -> String {
            "users".into()
        }
    }

    #[derive(Debug, Clone, Default)]
    struct Order {
        id: i64,
        user_id: i64,
        amount: f64,
    }

    impl Model for Order {
        fn table_name() -> String {
            "orders".into()
        }
    }

    #[derive(Debug, Clone, Default)]
    struct OrderItem {
        id: i64,
        order_id: i64,
        product_id: i64,
    }

    impl Model for OrderItem {
        fn table_name() -> String {
            "order_items".into()
        }
    }

    #[derive(Debug, Clone, Default)]
    struct Product {
        id: i64,
        name: String,
    }

    impl Model for Product {
        fn table_name() -> String {
            "products".into()
        }
    }

    impl Association<User> for Order {}

    impl Association<Order> for User {
        fn association_type() -> AssociationType {
            AssociationType::HasMany
        }
    }

    impl Association<Order> for OrderItem {}

    impl Association<OrderItem> for Order {
        fn association_type() -> AssociationType {
            AssociationType::HasMany
        }
    }

    impl Association<Product> for OrderItem {}
    impl Association<OrderItem> for Product {
        fn association_type() -> AssociationType {
            AssociationType::HasMany
        }
    }

    impl FromRow for User {
        fn from_row(row: Row) -> Self {
            let id: i64 = row.get("id");
            let email: String = row.get("email");
            let password: String = row.get("password");

            User {
                id,
                email,
                password,
            }
        }
    }

    impl FromRow for Order {
        fn from_row(row: Row) -> Self {
            let id: i64 = row.get("id");
            let user_id: i64 = row.get("user_id");
            let amount: f64 = row.get("amount");

            Order {
                id,
                user_id,
                amount,
            }
        }
    }

    impl FromRow for OrderItem {
        fn from_row(row: Row) -> Self {
            let id: i64 = row.get("id");
            let order_id: i64 = row.get("order_id");
            let product_id: i64 = row.get("product_id");

            OrderItem {
                id,
                order_id,
                product_id,
            }
        }
    }

    impl FromRow for Product {
        fn from_row(row: Row) -> Self {
            let id: i64 = row.get("id");
            let name: String = row.get("name");

            Product { id, name }
        }
    }

    #[test]
    fn test_join() {
        let query = User::all().join::<Order>().first_one();

        assert_eq!(
            query.to_sql(),
            r#"SELECT "users".* FROM "users" INNER JOIN "orders" ON "users"."id" = "orders"."user_id" ORDER BY "users"."id" ASC LIMIT 1"#
        );

        let query = Order::all().join::<User>();
        assert_eq!(
            query.to_sql(),
            r#"SELECT "orders".* FROM "orders" INNER JOIN "users" ON "orders"."user_id" = "users"."id""#
        );

        // Order that have a user with id = 5.
        let query = Order::all()
            .join::<User>()
            .find_by(User::column("id"), 5_i64.to_value());

        assert_eq!(
            query.to_sql(),
            r#"SELECT "orders".* FROM "orders" INNER JOIN "users" ON "orders"."user_id" = "users"."id" WHERE "users"."id" = $1 LIMIT 1"#
        );

        let query = User::all()
            .join::<Order>()
            .filter("id", 5)
            .filter(Order::column("amount"), 42.0);

        assert_eq!(
            query.to_sql(),
            r#"SELECT "users".* FROM "users" INNER JOIN "orders" ON "users"."id" = "orders"."user_id" WHERE "users"."id" = $1 AND "orders"."amount" = $2"#
        );

        let query = User::all()
            .join::<Order>()
            .join_nested(Order::join::<OrderItem>().join::<Product>())
            .filter(Product::column("name"), "test_product");

        println!("{}", query.to_sql());
    }

    #[test]
    fn test_related() {
        let query = User::related::<Order>([1, 2].as_slice());
        println!("{}", query.to_sql());
    }

    #[test]
    fn test_take_one() {
        let query = User::take_one().to_sql();

        assert_eq!(query, r#"SELECT * FROM "users" LIMIT 1"#);
    }

    #[test]
    fn test_take_many() {
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
        let query = User::filter("email", "test@test.com")
            .filter("password", "not_encrypted")
            .filter("id", 5)
            .filter("id", [1_i64, 2, 3].as_slice());

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
    async fn test_fetch() -> Result<(), Error> {
        let pool = Pool::new_local();
        let transaction = pool.begin().await?;

        transaction
            .query(
                "CREATE TABLE users (id BIGINT, email VARCHAR, password VARCHAR);",
                &[],
            )
            .await?;
        transaction
            .query(
                "INSERT INTO users VALUES (1, 'test@test.com', 'not_encrypted');",
                &[],
            )
            .await?;

        let user = User::order(("email", "ASC"))
            .first_one()
            .fetch(&transaction)
            .await?;

        assert_eq!(user.email, "test@test.com");

        let users = User::all().fetch_all(&transaction).await?;

        assert_eq!(users.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_explain() -> Result<(), Error> {
        let pool = Pool::new_local();
        let transaction = pool.begin().await?;

        transaction
            .execute("CREATE TABLE users (id BIGINT);", &[])
            .await?;

        let explain = User::all().explain(&transaction).await?;
        assert!(explain.to_string().starts_with("Seq Scan on users"));

        Ok(())
    }

    // #[test]
    // fn test_or() {
    //     let query = User::all()
    //         .filter(&[("email", "test@test.com")])
    //         .filter(&[("password", "not_encrypted")])
    //         .or(User::all().filter(&[("email", "another@test.com")]));

    //     assert_eq!(
    //         query.to_sql(),
    //         r#"SELECT * FROM "users" WHERE ("users"."email" = $1 AND "users"."password" = $2) OR ("users"."email" = $3)"#
    //     );

    //     let query = User::all()
    //         .not(&[("email", "test@test.com")])
    //         .or_not(&[("email", "another@test.com")]);

    //     assert_eq!(
    //         query.to_sql(),
    //         r#"SELECT * FROM "users" WHERE ("users"."email" <> $1) OR ("users"."email" <> $2)"#
    //     );
    // }
}
