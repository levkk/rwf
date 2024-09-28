use colored::Colorize;

use std::time::{Duration, Instant};
use tracing::info;

pub mod callbacks;
pub mod column;
pub mod error;
pub mod escape;
pub mod exists;
pub mod explain;
pub mod filter;
pub mod insert;
pub mod join;
pub mod limit;
pub mod lock;
pub mod macros;
pub mod migrations;
pub mod order_by;
pub mod placeholders;
pub mod pool;
pub mod row;
pub mod select;
pub mod update;
pub mod value;

pub use column::{Column, Columns, ToColumn};
pub use error::Error;
pub use escape::Escape;
pub use exists::Exists;
pub use explain::Explain;
pub use filter::{Filter, WhereClause};
pub use insert::Insert;
pub use join::{Association, AssociationType, Join, Joined, Joins};
pub use limit::Limit;
pub use lock::Lock;
pub use macros::belongs_to;
pub use migrations::{migrate, rollback};
pub use order_by::{OrderBy, OrderColumn, ToOrderBy};
pub use placeholders::Placeholders;
pub use pool::{get_connection, get_pool, Connection, ConnectionGuard, Pool};
pub use row::Row;
pub use select::Select;
pub use update::Update;
pub use value::{ToValue, Value};

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
pub trait FromRow: Clone + Send {
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
/// # Examples
///
/// #### Custom query
///
/// ```
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
///
/// let query = SelectUser { email: "test@test.com".into() }.to_sql();
/// assert_eq!(query, r"SELECT * FROM users WHERE email = 'test@test.com'");
/// ```
///
/// #### Equality check
///
/// ```
/// use rum::model::{ToSql, Escape};
///
/// struct Equals {
///     column: String,
///     value: i64,
/// }
///
/// impl ToSql for Equals {
///     fn to_sql(&self) -> String {
///         format!("{} = {}", self.column.escape(), self.value)
///     }
/// }
///
/// let equals = Equals { column: "id".into(), value: 5 };
/// assert_eq!(equals.to_sql(), "id = 5");
/// ```
pub trait ToSql {
    /// Convert `self` into a valid SQL entity.
    fn to_sql(&self) -> String;
}

/// The ORM query builder. All queries are constructed using this enum.
#[derive(Debug)]
pub enum Query<T: FromRow + ?Sized = Row> {
    Select(Select<T>),
    Update(Update<T>),
    Insert(Insert<T>),
    InsertIfNotExists {
        select: Select<T>,
        insert: Insert<T>,
        created: bool,
    },
    Raw(String),
}

impl<T: FromRow> ToSql for Query<T> {
    fn to_sql(&self) -> String {
        use Query::*;

        match self {
            Select(select) => select.to_sql(),
            Raw(query) => query.clone(),
            Update(update) => update.to_sql(),
            Insert(insert) => insert.to_sql(),
            InsertIfNotExists { select, insert, .. } => {
                format!("{}; {};", select.to_sql(), insert.to_sql())
            }
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
    /// use rum::model::{Query, ToSql, Row};
    ///
    /// let query = Query::<Row>::select("users");
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
    /// use rum::model::{Query, ToSql, Row};
    ///
    /// let query = Query::<Row>::select("users").take_one();
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
    /// use rum::model::{Query, ToSql, Row};
    ///
    /// let query = Query::<Row>::select("users").take_many(25);
    /// assert_eq!(query.to_sql(), "SELECT * FROM \"users\" LIMIT 25");
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

    pub fn filter_gt(self, column: impl ToColumn, value: impl ToValue) -> Self {
        use Query::*;
        match self {
            Select(select) => Select(select.filter_gt(column, value)),
            _ => self,
        }
    }

    pub fn filter_gte(self, column: impl ToColumn, value: impl ToValue) -> Self {
        use Query::*;
        match self {
            Select(select) => Select(select.filter_gte(column, value)),
            _ => self,
        }
    }

    pub fn filter_lt(self, column: impl ToColumn, value: impl ToValue) -> Self {
        use Query::*;
        match self {
            Select(select) => Select(select.filter_lt(column, value)),
            _ => self,
        }
    }

    pub fn filter_lte(self, column: impl ToColumn, value: impl ToValue) -> Self {
        use Query::*;
        match self {
            Select(select) => Select(select.filter_lte(column, value)),
            _ => self,
        }
    }

    pub fn or(self, f: fn(Self) -> Self) -> Self {
        use Query::*;
        match self {
            Select(mut select) => {
                let or = select.or();
                let query = f(Select(or));
                match query {
                    Select(or) => {
                        select.where_clause.or(or.where_clause.filter());
                        select.placeholders = or.placeholders;
                        Select(select)
                    }

                    _ => Select(select),
                }
            }
            _ => self,
        }
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
    /// use rum::{model::{Association, Model}, macros::Model};
    ///
    /// #[derive(Clone, Default, Model)]
    /// struct User {}
    ///
    /// #[derive(Clone, Default, Model)]
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

    pub fn lock(self) -> Self {
        match self {
            Query::Select(select) => Query::Select(select.lock()),
            _ => self,
        }
    }

    pub fn skip_locked(self) -> Self {
        match self {
            Query::Select(select) => Query::Select(select.skip_locked()),
            _ => self,
        }
    }

    pub fn find_or_create(self) -> Self {
        match self {
            Query::Select(select) => {
                let (columns, values) = select.insert_columns();
                let insert = Insert::from_columns(&columns, &values);
                Query::InsertIfNotExists {
                    select,
                    insert,
                    created: false,
                }
            }
            _ => self,
        }
    }

    pub fn update_all(self, attributes: &[(impl ToColumn, impl ToValue)]) -> Self {
        match self {
            Query::Select(select) => {
                let update = Update::<T>::from(select);
                let columns = attributes
                    .iter()
                    .map(|(c, _)| c.to_column())
                    .collect::<Vec<_>>();
                let values = attributes
                    .iter()
                    .map(|(_, v)| v.to_value())
                    .collect::<Vec<_>>();
                Query::Update(update.columns(&columns, &values))
            }
            _ => self,
        }
    }

    async fn execute_internal(
        &self,
        client: &mut ConnectionGuard,
    ) -> Result<Vec<tokio_postgres::Row>, Error> {
        let rows = match self {
            Query::Select(select) => {
                let query = self.to_sql();
                let placeholdres = { select.placeholders() };
                let values = placeholdres.values();
                client.query_cached(&query, &values).await?
            }

            Query::Raw(query) => client.query_cached(query, &[]).await?,

            Query::Update(update) => {
                let query = self.to_sql();
                let values = update.placeholders.values();
                client.query_cached(&query, &values).await?
            }

            Query::Insert(insert) => {
                let query = self.to_sql();
                let values = insert.placeholders.values();
                client.query_cached(&query, &values).await?
            }

            Query::InsertIfNotExists { select, insert, .. } => {
                let query = select.to_sql();
                let values = select.placeholders().values();
                let result = client.query_cached(&query, &values).await?;

                if result.is_empty() {
                    let query = insert.to_sql();
                    let values = insert.placeholders.values();
                    client.query_cached(&query, &values).await?
                } else {
                    result
                }
            }
        };

        Ok(rows)
    }

    /// Execute the query and fetch the first row from the database.
    pub async fn fetch(self, conn: &mut ConnectionGuard) -> Result<T, Error> {
        match self.execute(conn).await?.first().cloned() {
            Some(row) => Ok(row),
            None => Err(Error::RecordNotFound),
        }
    }

    pub async fn fetch_optional(self, conn: &mut ConnectionGuard) -> Result<Option<T>, Error> {
        match self.fetch(conn).await {
            Ok(row) => Ok(Some(row)),
            Err(Error::RecordNotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Execute the query and fetch all rows from the database.
    pub async fn fetch_all(self, conn: &mut ConnectionGuard) -> Result<Vec<T>, Error> {
        self.execute(conn).await
    }

    /// Get the query plan from Postgres.
    ///
    /// Take the actual query, prepend `EXPLAIN` and execute.
    pub async fn explain(self, conn: &mut ConnectionGuard) -> Result<Explain, Error> {
        let query = Query::<Explain>::Raw(format!("EXPLAIN {}", self.to_sql()));
        match query.execute_internal(conn).await?.pop() {
            Some(explain) => Ok(Explain::from_row(explain)),
            None => Err(Error::RecordNotFound),
        }
    }

    pub async fn exists(self, conn: &mut ConnectionGuard) -> Result<bool, Error> {
        Ok(self.count(conn).await? > 0)
    }

    pub async fn count(self, conn: &mut ConnectionGuard) -> Result<i64, Error> {
        let query = match self {
            Query::Select(select) => Query::Select(select.exists()),
            _ => self,
        };
        let start = Instant::now();

        let result = match query.execute_internal(conn).await?.pop() {
            None => Ok(0),
            Some(exists) => Ok(Exists::from_row(exists).count),
        };

        query.log(start.elapsed());

        result
    }

    /// Execute a query and return an optional result.
    pub async fn execute(self, conn: &mut ConnectionGuard) -> Result<Vec<T>, Error> {
        let start = Instant::now();
        let result = self
            .execute_internal(conn)
            .await?
            .into_iter()
            .map(|row| T::from_row(row))
            .collect();
        let time = start.elapsed();

        self.log(time);

        Ok(result)
    }

    fn log(&self, duration: Duration) {
        info!(
            "{} {} ({:.3} ms) {}",
            std::any::type_name::<T>()
                .split("::")
                .skip(1)
                .collect::<Vec<_>>()
                .join("::")
                .green(),
            match self {
                Query::Select(_) => "load".purple(),
                Query::Update(_) => "save".purple(),
                Query::Raw(_) => "query".purple(),
                Query::Insert(_) => "save".purple(),
                Query::InsertIfNotExists { .. } => "load/create".purple(),
            },
            duration.as_secs_f64() * 1000.0,
            self.to_sql()
        );
    }
}

pub type Scope<T> = Query<T>;

pub trait Model: FromRow {
    /// Name of the Postgres table.
    ///
    /// Typically this is automatically inferred based on the name of the struct,
    /// if you're using the derive macro. If not, you can specify any table name you want.
    fn table_name() -> String;

    /// When joining tables, use this function to create a fully-qualified column name, e.g.
    /// instead "id", you'll get "users"."id".
    fn column(name: &str) -> Column {
        Column::new(Self::table_name(), name)
    }

    /// List of columns in the table.
    ///
    /// If you're using the derive macro, you don't need to specify these,
    /// they will be inferred from the struct attributes.
    fn column_names() -> &'static [&'static str];

    /// The value of the primary key (id).
    ///
    /// All models require an ID field. This makes things a lot easier for
    /// not only day-to-day operations but also joins.
    ///
    /// This method is implemented if the derive macro is used.
    /// Otherwise, it should return the value of the `id` struct attribute.
    fn id(&self) -> Value;

    /// Values is a list of all column values as mapped to the struct attributes.
    ///
    /// Should be in the same order as the [`Self::column_names`].
    fn values(&self) -> Vec<Value>;

    /// If this table is related to another table, this is the name of the foreign key.
    ///
    /// For example, if the primary key of this table is "id" and the name of the table is "users",
    /// then the foreign key is "user_id".
    fn foreign_key() -> String;

    /// Name of the primary key column. Expected to be "id".
    fn primary_key() -> String {
        "id".to_string()
    }

    /// `LIMIT 1`
    fn take_one() -> Query<Self> {
        Query::select(Self::table_name()).take_one()
    }

    /// `LIMIT n`
    fn take_many(n: usize) -> Query<Self> {
        Query::select(Self::table_name()).take_many(n)
    }

    /// `ORDER BY id ASC LIMIT 1`
    fn first_one() -> Query<Self> {
        Query::select(Self::table_name()).first_one()
    }

    /// `ORDER BY id ASC LIMIT n`
    fn first_many(n: usize) -> Query<Self> {
        Query::select(Self::table_name()).first_many(n)
    }

    /// Get all rows. Good starting point for all queries.
    fn all() -> Query<Self> {
        Query::select(Self::table_name())
    }

    /// `WHERE column = value`
    fn filter(column: impl ToColumn, value: impl ToValue) -> Query<Self> {
        Query::select(Self::table_name()).filter(column, value)
    }

    /// `WHERE column = value LIMIT 1`
    fn find_by(column: impl ToColumn, value: impl ToValue) -> Query<Self> {
        Query::select(Self::table_name())
            .find_by(column, value.to_value())
            .take_one()
    }

    /// `WHERE id = value`
    fn find(value: impl ToValue) -> Query<Self> {
        Query::select(Self::table_name())
            .find_by(Self::primary_key(), value.to_value())
            .take_one()
    }

    /// Whatever you want.
    fn find_by_sql(query: impl ToString) -> Query<Self> {
        Query::Raw(query.to_string())
    }

    fn order(order: impl ToOrderBy) -> Query<Self> {
        Self::all().order(order)
    }

    fn join<F: Association<Self>>() -> Joined<Self, F> {
        Joined::new(F::construct_join())
    }

    fn related<F: Association<Self>>(models: &[impl Model]) -> Query<F> {
        let fks = models
            .iter()
            .filter(|model| !model.id().is_null())
            .map(|fk| fk.id())
            .collect::<Vec<_>>();
        F::all().filter(Self::foreign_key(), fks.as_slice())
    }

    fn save(self) -> Query<Self> {
        match self.id().is_null() {
            false => Query::Update(Update::new(self)),
            true => Query::Insert(Insert::new(self)),
        }
    }

    fn create(self) -> Query<Self> {
        Query::Insert(Insert::new(self))
    }

    fn lock() -> Query<Self> {
        Self::all().lock()
    }

    fn to_json(&self) -> Result<serde_json::Value, Error> {
        let columns = Self::column_names();
        let values = self.values();

        let mut map = serde_json::Map::new();
        for (column, value) in columns.iter().zip(values.iter()) {
            map.insert(column.to_string(), value.clone().into());
        }

        map.insert("id".into(), self.id().into());

        Ok(serde_json::Value::Object(map))
    }
}

#[cfg(test)]
mod test {
    use super::join::AssociationType;
    use super::*;
    use tokio_postgres::row::Row;

    #[derive(Debug, Clone, Default)]
    struct User {
        id: i64,
        email: String,
        password: String,
    }

    impl Model for User {
        fn id(&self) -> Value {
            Value::Integer(self.id)
        }

        fn table_name() -> String {
            "users".into()
        }

        fn foreign_key() -> String {
            "user_id".into()
        }

        fn column_names() -> &'static [&'static str] {
            &["email", "password"]
        }

        fn values(&self) -> Vec<Value> {
            vec![self.email.to_value(), self.password.to_value()]
        }
    }

    #[derive(Debug, Clone, Default)]
    struct Order {
        id: i64,
        user_id: i64,
        amount: f64,
    }

    impl Model for Order {
        fn id(&self) -> Value {
            Value::Integer(self.id)
        }

        fn table_name() -> String {
            "orders".into()
        }

        fn foreign_key() -> String {
            "order_id".into()
        }

        fn column_names() -> &'static [&'static str] {
            &["user_id", "amount"]
        }

        fn values(&self) -> Vec<Value> {
            vec![self.user_id.to_value(), self.amount.to_value()]
        }
    }

    #[derive(Debug, Clone, Default)]
    struct OrderItem {
        id: i64,
        order_id: i64,
        product_id: i64,
    }

    impl Model for OrderItem {
        fn id(&self) -> Value {
            Value::Integer(self.id)
        }

        fn table_name() -> String {
            "order_items".into()
        }

        fn foreign_key() -> String {
            "order_item_id".into()
        }

        fn column_names() -> &'static [&'static str] {
            &["order_id", "product_id"]
        }

        fn values(&self) -> Vec<Value> {
            vec![self.order_id.to_value(), self.product_id.to_value()]
        }
    }

    #[derive(Debug, Clone, Default)]
    struct Product {
        id: i64,
        name: String,
    }

    impl Model for Product {
        fn id(&self) -> Value {
            Value::Integer(self.id)
        }

        fn table_name() -> String {
            "products".into()
        }

        fn foreign_key() -> String {
            "product_id".into()
        }

        fn column_names() -> &'static [&'static str] {
            &["name"]
        }

        fn values(&self) -> Vec<Value> {
            vec![self.name.to_value()]
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
        // let query = User::related::<Order>([1, 2].as_slice());
        // println!("{}", query.to_sql());
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
        let mut transaction = pool.begin().await?;

        transaction.client().query("DROP TABLE users", &[]).await?;

        transaction
            .client()
            .query(
                "CREATE TABLE IF NOT EXISTS users (id BIGINT, email VARCHAR, password VARCHAR);",
                &[],
            )
            .await?;
        transaction
            .client()
            .query(
                "INSERT INTO users VALUES (1, 'test@test.com', 'not_encrypted');",
                &[],
            )
            .await?;

        let user = User::order(("email", "ASC"))
            .first_one()
            .fetch(&mut transaction)
            .await?;

        assert_eq!(user.email, "test@test.com");

        let users = User::all().fetch_all(&mut transaction).await?;

        assert_eq!(users.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_explain() -> Result<(), Error> {
        let pool = Pool::new_local();
        let mut transaction = pool.begin().await?;

        transaction
            .client()
            .execute("CREATE TABLE IF NOT EXISTS users (id BIGINT);", &[])
            .await?;

        let explain = User::all().explain(&mut transaction).await?;
        assert!(explain.to_string().starts_with("Seq Scan on users"));

        Ok(())
    }

    #[tokio::test]
    async fn test_find_or_create() -> Result<(), Error> {
        let pool = Pool::new_local();

        let mut transaction = pool.begin().await?;

        transaction
            .client()
            .execute("DROP TABLE IF EXISTS users", &[])
            .await?;
        transaction
            .client()
            .execute("CREATE TABLE IF NOT EXISTS users (id BIGSERIAL PRIMARY KEY, email VARCHAR NOT NULL, password VARCHAR NOT NULL);", &[])
            .await?;

        let query = User::all()
            .filter("email", "test@test.com")
            .filter("password", "password")
            .find_or_create();
        let sql = query.to_sql();
        assert_eq!(
            sql,
            r#"SELECT * FROM "users" WHERE "users"."email" = $1 AND "users"."password" = $2; INSERT INTO "users" ("email", "password") VALUES ($1, $2) RETURNING *;"#
        );

        query.fetch(&mut transaction).await?;

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
