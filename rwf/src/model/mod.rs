//! Object-relational mapper (ORM), the **M** in MVC.
//!
//! See [documentation](https://levkk.github.io/rwf/models/) for detailed examples on how to use the ORM.
use crate::colors::MaybeColorize;
use crate::config::get_config;

use pool::{ConnectionRequest, ToConnectionRequest};
use std::time::{Duration, Instant};
use tracing::{error, info};

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
pub mod migrations;
pub mod order_by;
pub mod picked;
pub mod placeholders;
pub mod pool;
pub mod prelude;
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
pub use migrations::{migrate, rollback, Migrations};
pub use order_by::{OrderBy, OrderColumn, ToOrderBy};
pub use picked::Picked;
pub use placeholders::Placeholders;
pub use pool::{get_connection, get_pool, start_transaction, Connection, ConnectionGuard, Pool};
pub use row::Row;
pub use select::Select;
pub use update::Update;
pub use value::{ToValue, Value};

/// Convert a PostgreSQL row to a Rust struct. Type conversions are handled by `tokio_postgres`. This only
/// creates a mapping between columns and struct fields.
///
/// This trait needs to be implemented for all structs that implement the [`Model`] trait.
///
///
/// # Example
///
/// ```
/// use rwf::model::{FromRow, Error};
///
/// #[derive(Clone)]
/// struct User {
///     id: i64,
///     email: String,
/// }
///
/// impl FromRow for User {
///     fn from_row(row: tokio_postgres::Row) -> Result<Self, Error> {
///         Ok(User {
///             id: row.try_get("id")?,
///             email: row.try_get("email")?
///         })
///     }
/// }
/// ```
pub trait FromRow: Clone + Send {
    /// Convert a [`tokio_postgres::Row`] to [`Self`].
    fn from_row(row: tokio_postgres::Row) -> Result<Self, Error>
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
/// This trait is used internally by the ORM to generate SQL queries.
///
/// # Examples
///
/// #### Custom query
///
/// ```
/// use rwf::model::{ToSql, Escape};
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
/// use rwf::model::{ToSql, Escape};
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

/// The query builder. It constructs the actual queries,
/// executes them against the database, and returns the results
/// converted into Rust types.
///
/// The query builder should not be instantiated directly. Use the [`Model`] trait instead,
/// implemented for your struct.
///
/// # Example
///
/// Let's define a struct implemeting the [`Model`] trait and use it as an example:
///
/// ```
/// # use rwf::macros::Model;
/// #[derive(Clone, Model)]
/// struct User {
///     id: Option<i64>,
///     email: String,
///     admin: bool,
/// }
/// ```
///
/// Calling any [`Model`] method, e.g., [`Model::all`], will return the query builder with the
/// specified Rust type:
///
/// ```
/// # use rwf::macros::Model;
/// # use rwf::model::{Model, Query, Select};
/// # #[derive(Clone, Debug, Model)]
/// # struct User {
/// #    id: Option<i64>,
/// #    email: String,
/// #    admin: bool,
/// # }
/// let query = User::all();
///
/// match query {
///     Query::Select(Select::<User> { table_name, .. }) => {
///         assert_eq!(table_name, User::table_name());
///     }
///     _ => unreachable!(),
/// }
/// ```
///
/// By itself the query builder object will not execute the query until [`Query::execute`] is called. This allows the
/// query builder to be passed around the code base and modified depending on the context.
///
/// ## Scopes
///
/// Since the query builder represents a unexecuted query, it can be saved for later re-use. This allows us to implement
/// "scopes", re-usable queries that can be modified or executed as-is. To make this more user-friendly, the [`Scope`]
/// type is aliased to [`Query`], for example:
///
/// ```
/// # use rwf::macros::Model;
/// # use rwf::model::{Model, Query, Select, Scope};
/// # #[derive(Clone, Debug, Model)]
/// # struct User {
/// #    id: Option<i64>,
/// #    email: String,
/// #    admin: bool,
/// # }
/// impl User {
///     fn admins() -> Scope<User> {
///         Self::filter("admin", true)
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub enum Query<T: FromRow + ?Sized = Row> {
    /// Represents a `SELECT` query.
    Select(Select<T>),
    /// Represents an `UPDATE` statement.
    Update(Update<T>),
    /// Represents an `INSERT` statement.
    Insert(Insert<T>),
    /// Implements [`Model::find_or_create_by`] by building a `SELECT` and an `INSERT` query.
    InsertIfNotExists {
        select: Select<T>,
        insert: Insert<T>,
        created: bool,
    },
    /// An arbitrary query.
    Raw {
        query: String,
        placeholders: Placeholders,
    },
    /// **WIP**: `SELECT` query with only specific columns.
    Picked(Picked<T>),
}

impl<T: FromRow> ToSql for Query<T> {
    fn to_sql(&self) -> String {
        use Query::*;

        match self {
            Select(select) => select.to_sql(),
            Raw { query, .. } => query.clone(),
            Update(update) => update.to_sql(),
            Insert(insert) => insert.to_sql(),
            InsertIfNotExists { select, insert, .. } => {
                format!("{}; {};", select.to_sql(), insert.to_sql())
            }
            Picked(picked) => picked.select.to_sql(),
        }
    }
}

impl<T: Model> Query<T> {
    /// Start a `SELECT` query for the given table. This method is mostly used internally.
    ///
    /// # Arguments
    ///
    /// * `table_name` - The name of the database table.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::macros::Model;
    /// # use rwf::model::{Model, Query, Select, ToSql};
    /// # #[derive(Clone, Debug, Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// #    admin: bool,
    /// # }
    /// let query = Query::<User>::select("users");
    /// assert_eq!(query.to_sql(), r#"SELECT * FROM "users""#);
    /// ```
    pub fn select(table_name: impl ToString) -> Self {
        Query::Select(Select::new(
            table_name.to_string().as_str(),
            &T::primary_key(),
        ))
    }

    /// Create a query that selects one row from the relation. The rows are not ordered and any row can be returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::macros::Model;
    /// # use rwf::model::{Model, Query, Select, ToSql};
    /// # #[derive(Clone, Debug, Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// #    admin: bool,
    /// # }
    /// let query = User::all().take_one();
    /// assert_eq!(query.to_sql(), r#"SELECT * FROM "users" LIMIT 1"#);
    /// ```
    pub fn take_one(self) -> Self {
        use Query::*;

        match self {
            Select(select) => Select(select.limit(1)),
            _ => self,
        }
    }

    /// Create a query that selects _n_ rows from the relation. The order of rows is determined by the database.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::macros::Model;
    /// # use rwf::model::{Model, Query, Select, ToSql};
    /// # #[derive(Clone, Debug, Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// #    admin: bool,
    /// # }
    /// let query = User::all().take_many(25);
    /// assert_eq!(query.to_sql(), r#"SELECT * FROM "users" LIMIT 25"#);
    /// ```
    pub fn take_many(self, n: i64) -> Self {
        use Query::*;

        match self {
            Select(select) => Select(select.limit(n)),
            _ => self,
        }
    }

    /// Create a query that selects the first row from the database. The rows are ordered
    ///  by primary key in ascending order.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::macros::Model;
    /// # use rwf::model::{Model, Query, Select, ToSql};
    /// # #[derive(Clone, Debug, Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// #    admin: bool,
    /// # }
    /// let query = User::all().first_one();
    /// assert_eq!(query.to_sql(), r#"SELECT * FROM "users" ORDER BY "users"."id" ASC LIMIT 1"#);
    /// ```
    pub fn first_one(self) -> Self {
        use Query::*;

        match self {
            Select(_) => self.first_many(1),
            _ => self,
        }
    }

    /// Creates a query that selects first _n_ rows from the database. Rows are sorted
    /// by primary key in ascending order.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::macros::Model;
    /// # use rwf::model::{Model, Query, Select, ToSql};
    /// # #[derive(Clone, Debug, Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// #    admin: bool,
    /// # }
    /// let query = User::all().first_many(25);
    /// assert_eq!(query.to_sql(), r#"SELECT * FROM "users" ORDER BY "users"."id" ASC LIMIT 25"#);
    /// ```
    pub fn first_many(self, n: i64) -> Self {
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

            _ => self,
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

    pub fn filter_not(self, column: impl ToColumn, value: impl ToValue) -> Self {
        self.not(column, value)
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

    pub fn limit(self, limit: i64) -> Self {
        self.take_many(limit)
    }

    pub fn offset(self, offset: i64) -> Self {
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
    /// use rwf::{model::{Association, Model}, macros::Model};
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
                    select: select.limit(1),
                    insert,
                    created: false,
                }
            }
            _ => self,
        }
    }

    pub fn column(self, column: impl ToColumn) -> Self {
        match self {
            Query::Select(select) => {
                let select = select.select_additional(column);
                Query::Select(select)
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

    pub fn unique_by(self, columns: &[impl ToColumn]) -> Self {
        match self {
            Query::Insert(insert) => Query::Insert(insert.unique_by(columns)),
            Query::InsertIfNotExists {
                select,
                insert,
                created,
            } => {
                let insert = insert.unique_by(columns);
                Query::InsertIfNotExists {
                    select,
                    insert,
                    created,
                }
            }
            _ => self,
        }
    }

    async fn execute_internal(
        &self,
        client: impl ToConnectionRequest<'_>,
    ) -> Result<Vec<tokio_postgres::Row>, Error> {
        let request = client.to_connection_request()?;
        let mut conn = request.get().await?;

        let client = match request.connection() {
            Some(conn) => conn,
            None => conn.as_mut().unwrap(),
        };

        let result = match self {
            Query::Select(select) => {
                let query = self.to_sql();
                let placeholdres = { select.placeholders() };
                let values = placeholdres.values();
                client.query_cached(&query, &values).await
            }

            Query::Raw {
                query,
                placeholders,
            } => {
                let values = placeholders.values();
                client.query_cached(query, &values).await
            }

            Query::Update(update) => {
                let query = self.to_sql();
                let values = update.placeholders.values();
                client.query_cached(&query, &values).await
            }

            Query::Insert(insert) => {
                let query = self.to_sql();
                let values = insert.placeholders.values();
                client.query_cached(&query, &values).await
            }

            Query::InsertIfNotExists { select, insert, .. } => {
                let query = select.to_sql();
                let values = select.placeholders().values();
                let result = client.query_cached(&query, &values).await;

                let result = match result {
                    Ok(result) => result,
                    Err(err) => {
                        self.log_error(&err);
                        return Err(err);
                    }
                };

                if result.is_empty() {
                    let query = insert.to_sql();
                    let values = insert.placeholders.values();
                    client.query_cached(&query, &values).await
                } else {
                    Ok(result)
                }
            }

            Query::Picked(picked) => {
                let select = &picked.select;
                let query = select.to_sql();
                let placeholdres = { select.placeholders() };
                let values = placeholdres.values();
                client.query_cached(&query, &values).await
            }
        };

        match result {
            Ok(rows) => Ok(rows),
            Err(err) => {
                self.log_error(&err);
                Err(err)
            }
        }
    }

    /// Execute the query and fetch the first row from the database.
    pub async fn fetch(self, conn: impl ToConnectionRequest<'_>) -> Result<T, Error> {
        match self.execute(conn).await?.first().cloned() {
            Some(row) => Ok(row),
            None => Err(Error::RecordNotFound),
        }
    }

    pub async fn fetch_optional(
        self,
        conn: impl ToConnectionRequest<'_>,
    ) -> Result<Option<T>, Error> {
        match self.fetch(conn).await {
            Ok(row) => Ok(Some(row)),
            Err(Error::RecordNotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Execute the query and fetch all rows from the database.
    pub async fn fetch_all(self, conn: impl ToConnectionRequest<'_>) -> Result<Vec<T>, Error> {
        self.execute(conn).await
    }

    /// Get the query plan from Postgres.
    ///
    /// Take the actual query, prepend `EXPLAIN` and execute.
    pub async fn explain(self, conn: impl ToConnectionRequest<'_>) -> Result<Explain, Error> {
        let query = format!("EXPLAIN {}", self.to_sql());
        let placeholders = match self {
            Query::Select(select) => select.placeholders,
            Query::Update(update) => update.placeholders,
            Query::Insert(insert) => insert.placeholders,
            Query::Picked(picked) => picked.select.placeholders,
            _ => todo!("explain"),
        };

        let query = Query::<Explain>::Raw {
            query,
            placeholders,
        };
        match query.execute_internal(conn).await?.pop() {
            Some(explain) => Ok(Explain::from_row(explain)?),
            None => Err(Error::RecordNotFound),
        }
    }

    pub async fn exists(self, conn: impl ToConnectionRequest<'_>) -> Result<bool, Error> {
        Ok(self.count(conn).await? > 0)
    }

    pub async fn count(self, conn: impl ToConnectionRequest<'_>) -> Result<i64, Error> {
        let query = match self {
            Query::Select(select) => Query::Select(select.exists()),
            _ => self,
        };
        let start = Instant::now();

        let result = match query.execute_internal(conn).await?.pop() {
            None => Ok(0),
            Some(exists) => Ok(Exists::from_row(exists)?.count),
        };

        query.log(start.elapsed());

        result
    }

    /// Execute a query and return an optional result.
    pub async fn execute(self, conn: impl ToConnectionRequest<'_>) -> Result<Vec<T>, Error> {
        let start = Instant::now();
        let mut results = vec![];
        let rows = self.execute_internal(conn).await?;
        for row in rows {
            results.push(T::from_row(row)?)
        }
        let time = start.elapsed();

        self.log(time);

        Ok(results)
    }

    fn type_name() -> String {
        std::any::type_name::<T>()
            .split("::")
            .skip(1)
            .collect::<Vec<_>>()
            .join("::")
    }

    fn action(&self) -> &'static str {
        match self {
            Query::Select(_) | Query::Picked(_) => "load",
            Query::Update(_) => "save",
            Query::Raw { .. } => "query",
            Query::Insert(_) => "save",
            Query::InsertIfNotExists { .. } => "load/create",
        }
    }

    fn log(&self, duration: Duration) {
        if !get_config().general.log_queries {
            return;
        }

        info!(
            "{} {} ({:.3} ms) {}",
            Self::type_name().green(),
            self.action().purple(),
            duration.as_secs_f64() * 1000.0,
            self.to_sql()
        );
    }

    fn log_error(&self, err: &Error) {
        error!(
            "{} {} {} {}",
            Self::type_name().green(),
            self.action().purple(),
            self.to_sql(),
            err,
        )
    }
}

pub type Scope<T> = Query<T>;

/// Implements Object-relational mapping (ORM) methods for a Rust struct, turning it into a database model. This trait doesn't have to be implemented manually. You can use the [`rwf_macros::Model`] macro instead, for example:
///
/// ```
/// # use rwf::prelude::*;
/// #[derive(Clone, macros::Model)]
/// struct User {
///     id: Option<i64>,
///     email: String,
/// }
/// ```
///
/// Structs that wish to implement this trait need to implement two additional traits:
///
/// - [`FromRow`]
/// - [`Clone`]
///
/// When usindg the [`rwf_macros::Model`] derive, [`FromRow`] is derived automatically.
pub trait Model: FromRow {
    /// Name of the PostgreSQL table where records for this model are stored.
    ///
    /// The name must not be fully qualified
    ///  e.g. `"users"` is correct, while `r#"public"."users"#` won't work.
    ///
    /// This method is implemented automatically by the [`rwf_macros::Model`] derive. If you wish to override
    /// that implementation, use `#[table_name("your_name")]` derive attribute.
    fn table_name() -> &'static str;

    /// Get the fully qualified column name for this table.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::prelude::*;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// let column = User::column("email");
    ///
    /// assert_eq!(column.to_string(), r#""users"."email""#);
    /// ```
    fn column(name: &str) -> Column {
        Column::new(Self::table_name(), name)
    }

    /// List of names of columns stored in the PostgreSQL table.
    ///
    /// Names must not be fully qualified or contain
    /// double quotes, e.g. `"id"` is correct, while `"users"."id"` won't work.
    ///
    /// This method is implemented automatically by the [`rwf_macros::Model`] derive.
    ///
    /// # Example
    /// ```
    /// fn column_names() -> &'static [&'static str] {
    ///     &["id", "email"]
    /// }
    /// ```
    fn column_names() -> &'static [&'static str];

    /// The primary key value, if one exists, for the instance of a model.
    ///
    /// Primary keys are not required to use the ORM, but are generally needed to perform
    /// updates and deletes. If the model doesn't have a primary key, you can return [`Value::Null`].
    ///
    /// This method is implemented automatically by the [`rwf_macros::Model`] derive.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::Value;
    /// # use rwf::macros::FromRow;
    /// # #[derive(Clone, FromRow)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// # impl Model for User {
    /// # fn table_name() -> &'static str { "" }
    /// # fn values(&self) -> Vec<Value> { vec![] }
    /// # fn column_names() -> &'static [&'static str] { &[] }
    /// # fn foreign_key() -> &'static str { "" }
    /// fn id(&self) -> Value {
    ///     self.id.to_value()
    /// }
    /// # }
    /// ```
    ///
    fn id(&self) -> Value;

    /// List of column values for a particular instance of a model. The values must be in the same
    /// order as the columns in [`Model::column_names`].
    ///
    /// The values are Rust types converted to the [`Value`] enum. This conversion is required so
    /// the ORM can then convert them to PostgreSQL types automatically. Most Rust types can be converted
    /// to [`Value`] automatically as well with [`ToValue::to_value`].
    ///
    /// This method is implemented automatically by the [`rwf_macros::Model`] derive.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::Value;
    /// # use rwf::macros::FromRow;
    /// # #[derive(Clone, FromRow)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// # impl Model for User {
    /// # fn table_name() -> &'static str { "" }
    /// # fn id(&self) -> Value { Value::Null }
    /// # fn column_names() -> &'static [&'static str] { &[] }
    /// # fn foreign_key() -> &'static str { "" }
    /// fn values(&self) -> Vec<Value> {
    ///     vec![
    ///         1_i64.to_value(),
    ///         "test@test.com".to_value(),
    ///     ]
    /// }
    /// # }
    /// ```
    fn values(&self) -> Vec<Value>;

    /// The name of a column in another PostgreSQL table which refers to this model.
    ///
    /// For example, if the table name for this model is `"users"`, this method could return `"user_id"`.
    ///
    /// This method is implemented automatically by the [`rwf_macros::Model`] derive. If you wish to override
    /// that implementation, use `#[foreign_key("your_fk")]` derive attribute.
    ///
    /// # Example
    ///
    /// ```
    /// fn foreign_key() -> &'static str {
    ///     "user_id"
    /// }
    /// ```
    fn foreign_key() -> &'static str;

    /// Name of the primary key column in the database.
    ///
    /// This is typically `"id"`, but can be any other column as long as it has a `UNIQUE NOT NULL` constraint
    /// and a default value produced from a sequence.
    fn primary_key() -> &'static str {
        "id"
    }

    /// Select one record from the table. The row returned is determined by the database.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// let query = User::take_one();
    ///
    /// assert_eq!(query.to_sql(), r#"SELECT * FROM "users" LIMIT 1"#);
    /// ```
    fn take_one() -> Query<Self> {
        Query::select(Self::table_name()).take_one()
    }

    /// Select _n_ records from the table. The order of rows returned is determined by the database.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// let query = User::take_many(25);
    ///
    /// assert_eq!(query.to_sql(), r#"SELECT * FROM "users" LIMIT 25"#);
    /// ```
    fn take_many(n: i64) -> Query<Self> {
        Query::select(Self::table_name()).take_many(n)
    }

    /// Select the first record from the table, ordered by primary key.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// let query = User::first_one();
    ///
    /// assert_eq!(query.to_sql(), r#"SELECT * FROM "users" ORDER BY "users"."id" ASC LIMIT 1"#);
    /// ```
    fn first_one() -> Query<Self> {
        Query::select(Self::table_name()).first_one()
    }

    /// Select the first _n_ records from the table, ordered by primary key.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// let query = User::first_many(25);
    ///
    /// assert_eq!(query.to_sql(), r#"SELECT * FROM "users" ORDER BY "users"."id" ASC LIMIT 25"#);
    /// ```
    fn first_many(n: i64) -> Query<Self> {
        Query::select(Self::table_name()).first_many(n)
    }

    /// Select all records from the table. Typically this is a starting point for filtering
    /// by some column(s), but all records can also be returned. Order of records returned is
    /// determined by the database, unless additional filters are specified.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// let query = User::all();
    ///
    /// assert_eq!(query.to_sql(), r#"SELECT * FROM "users""#);
    /// ```
    fn all() -> Query<Self> {
        Query::select(Self::table_name())
    }

    /// Filter table records by a column. Multiple calls to filter can be chained to filter by multiple columns.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// let query = User::filter("email", "test@test.com");
    ///
    /// assert_eq!(query.to_sql(), r#"SELECT * FROM "users" WHERE "users"."email" = $1"#);
    /// ```
    fn filter(column: impl ToColumn, value: impl ToValue) -> Query<Self> {
        Query::select(Self::table_name()).filter(column, value)
    }

    /// Filter table records by a column and fetch the first matching record. The record will match
    /// the filter, but if there are mulitple records that match, any one of them can be returned.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// let query = User::find_by("email", "test@test.com");
    ///
    /// assert_eq!(query.to_sql(), r#"SELECT * FROM "users" WHERE "users"."email" = $1 LIMIT 1"#);
    /// ```
    fn find_by(column: impl ToColumn, value: impl ToValue) -> Query<Self> {
        Query::select(Self::table_name())
            .find_by(column, value.to_value())
            .take_one()
    }

    /// Filter by primary key and return the matching row, if any.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// let query = User::find(1);
    ///
    /// assert_eq!(query.to_sql(), r#"SELECT * FROM "users" WHERE "users"."id" = $1 LIMIT 1"#);
    /// ```
    fn find(value: impl ToValue) -> Query<Self> {
        Query::select(Self::table_name())
            .find_by(Self::primary_key(), value.to_value())
            .take_one()
    }

    /// Find records using an arbitrary SQL query. The caller must ensure that all required columns
    /// are returned.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// let query = User::find_by_sql("SELECT * FROM users WHERE email = ANY($1, $2) ORDER BY RANDOM()",
    ///     &[
    ///         "bob@test.com".into(),
    ///         "alice@test.com".into(),
    ///     ]
    /// );
    ///
    /// assert_eq!(query.to_sql(), r#"SELECT * FROM users WHERE email = ANY($1, $2) ORDER BY RANDOM()"#);
    /// ```
    fn find_by_sql(query: impl ToString, values: &[Value]) -> Query<Self> {
        Query::Raw {
            query: query.to_string(),
            placeholders: values
                .iter()
                .map(|v| v.to_value())
                .collect::<Vec<_>>()
                .into(),
        }
    }

    /// Order records by a column. This method accepts any input type which implement
    /// the [`ToOrderBy`] trait. Chaining this function allows to order by multiple columns.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// let q1 = User::order("email");
    /// let q2 = User::order("email DESC");
    /// let q3 = User::order(("email", "DESC"));
    /// let q4 = User::order((User::column("email"), "DESC"));
    ///
    /// assert_eq!(q1.to_sql(), r#"SELECT * FROM "users" ORDER BY email"#);
    /// assert_eq!(q2.to_sql(), r#"SELECT * FROM "users" ORDER BY email DESC"#);
    /// assert_eq!(q3.to_sql(), r#"SELECT * FROM "users" ORDER BY "email" DESC"#);
    /// assert_eq!(q4.to_sql(), r#"SELECT * FROM "users" ORDER BY "users"."email" DESC"#);
    /// ```
    fn order(order: impl ToOrderBy) -> Query<Self> {
        Self::all().order(order)
    }

    /// Join this model to another model with which it has a relationship. The relationship should be declared
    /// in advance using an annotation.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// #[derive(Clone, macros::Model)]
    /// #[has_many(Project)]
    /// struct User {
    ///    id: Option<i64>,
    ///    email: String,
    /// }
    /// #[derive(Clone, macros::Model)]
    /// #[belongs_to(User)]
    /// struct Project {
    ///     user_id: i64,
    ///     name: String,
    /// }
    ///
    /// let join = User::join::<Project>();
    /// ```
    fn join<F: Association<Self>>() -> Joined<Self, F> {
        Joined::new(F::construct_join())
    }

    /// Filter all records which have a relationship to this model. Used for fetching multiple records at once
    /// in order to avoid N+1 queries.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// #[derive(Clone, macros::Model)]
    /// #[has_many(Project)]
    /// struct User {
    ///    id: Option<i64>,
    ///    email: String,
    /// }
    /// #[derive(Clone, macros::Model)]
    /// #[belongs_to(User)]
    /// struct Project {
    ///     user_id: i64,
    ///     name: String,
    /// }
    ///
    /// let alice = User { id: Some(1), email: "alice@test.com".into() };
    /// let bob = User { id: Some(2), email: "bob@test.com".into() };
    ///
    /// let projects = User::related::<Project>(&[alice, bob]);
    ///
    /// assert_eq!(
    ///     projects.to_sql(),
    ///     r#"SELECT * FROM "projects" WHERE "projects"."user_id" = ANY($1)"#
    /// );
    /// ```
    fn related<F: Association<Self>>(models: &[impl Model]) -> Query<F> {
        let fks = models
            .iter()
            .filter(|model| !model.id().is_null())
            .map(|fk| fk.id())
            .collect::<Vec<_>>();
        F::all().filter(Self::foreign_key(), fks.as_slice())
    }

    /// Save a model into the database. If a record already exists, it will be updated. If this is a new record,
    /// it will be inserted.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// // Save an existing user.
    /// let user = User { id: Some(1), email: "test@test.com".into() };
    /// assert_eq!(
    ///     user.save().to_sql(),
    ///     r#"UPDATE "users" SET "email" = $2 WHERE "id" = $1 RETURNING *"#,
    /// );
    ///
    /// // Create new user.
    /// let new_user = User { id: None, email: "alice@test.com".into() };
    /// assert_eq!(
    ///     new_user.save().to_sql(),
    ///     r#"INSERT INTO "users" ("email") VALUES ($1) RETURNING *"#,
    /// );
    /// ```
    fn save(self) -> Query<Self> {
        match self.id().is_null() {
            false => Query::Update(Update::new(self)),
            true => Query::Insert(Insert::new(self)),
        }
    }

    /// Create new record of this model. All columns that have a `NOT NULL` constraint and
    /// no default value should be provided.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// let user = User::create(&[
    ///     ("email", "bob@test.com"),
    /// ]);
    ///
    /// assert_eq!(
    ///     user.to_sql(),
    ///     r#"INSERT INTO "users" ("email") VALUES ($1) RETURNING *"#,
    /// );
    /// ```
    fn create(attributes: &[(impl ToColumn, impl ToValue)]) -> Query<Self> {
        let columns = attributes
            .iter()
            .map(|(c, _)| c.to_column())
            .collect::<Vec<_>>();
        let values = attributes
            .iter()
            .map(|(_, v)| v.to_value())
            .collect::<Vec<_>>();

        Query::Insert(Insert::from_columns(&columns, &values))
    }

    /// Find an existing record matching the column filters or create a new one
    /// if none already exist. It's is equivalent to running [`Model::filter`]
    /// and [`Model::create`] manually.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// User::find_or_create_by(&[
    ///     ("email", "john@test.com"),
    /// ]);
    /// ```
    fn find_or_create_by(attributes: &[(impl ToColumn, impl ToValue)]) -> Query<Self> {
        let columns = attributes
            .iter()
            .map(|(c, _)| c.to_column())
            .collect::<Vec<_>>();
        let values = attributes
            .iter()
            .map(|(_, v)| v.to_value())
            .collect::<Vec<_>>();

        let mut select = Query::<Self>::select(Self::table_name());

        for (column, value) in columns.iter().zip(values.iter()) {
            select = select.filter(column.clone(), value.clone());
        }

        let select = match select {
            Query::Select(select) => select,
            _ => unreachable!(),
        };

        let insert = Insert::<Self>::from_columns(&columns, &values);

        Query::InsertIfNotExists {
            select,
            insert,
            created: false,
        }
    }

    /// Lock all records for the duration of the transaction. No other transaction will be able
    /// to access those records until the current one is finished.
    ///
    /// It's not common to lock all rows in a table, so this function is typically chained with
    /// [`Model::filter`].
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    ///
    /// let lock = User::lock()
    ///     .filter("email", "test@test.com");
    ///
    /// assert_eq!(
    ///     lock.to_sql(),
    ///     r#"SELECT * FROM "users" WHERE "users"."email" = $1 FOR UPDATE"#,
    /// );
    /// ```
    fn lock() -> Query<Self> {
        Self::all().lock()
    }

    /// Refresh the record from the database. This is equivalent to calling [`Model::find`] with
    /// the primary key as the parameter.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// let user = User { id: Some(1), email: "test@test.com".into() };
    ///
    /// assert_eq!(
    ///     User::find(1).to_sql(),
    ///     user.reload().to_sql()
    /// );
    /// ```
    fn reload(self) -> Query<Self> {
        Self::find(self.id())
    }

    /// Convert the model to JSON representation.
    ///
    /// # Example
    /// ```
    /// # use rwf::prelude::*;
    /// # use rwf::model::ToSql;
    /// # #[derive(Clone, macros::Model)]
    /// # struct User {
    /// #    id: Option<i64>,
    /// #    email: String,
    /// # }
    /// use serde_json::json;
    ///
    /// let user = User { id: Some(1), email: "test@test.com".into() };
    ///
    /// assert_eq!(
    ///     user.to_json().unwrap(),
    ///     json!({
    ///         "id": 1,
    ///         "email": "test@test.com",
    ///     }),
    /// );
    /// ```
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

        fn table_name() -> &'static str {
            "users"
        }

        fn foreign_key() -> &'static str {
            "user_id"
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

        fn table_name() -> &'static str {
            "orders"
        }

        fn foreign_key() -> &'static str {
            "order_id"
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

        fn table_name() -> &'static str {
            "order_items"
        }

        fn foreign_key() -> &'static str {
            "order_item_id"
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

        fn table_name() -> &'static str {
            "products"
        }

        fn foreign_key() -> &'static str {
            "product_id"
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
        fn from_row(row: Row) -> Result<Self, Error> {
            let id: i64 = row.get("id");
            let email: String = row.get("email");
            let password: String = row.get("password");

            Ok(User {
                id,
                email,
                password,
            })
        }
    }

    impl FromRow for Order {
        fn from_row(row: Row) -> Result<Self, Error> {
            let id: i64 = row.get("id");
            let user_id: i64 = row.get("user_id");
            let amount: f64 = row.get("amount");

            Ok(Order {
                id,
                user_id,
                amount,
            })
        }
    }

    impl FromRow for OrderItem {
        fn from_row(row: Row) -> Result<Self, Error> {
            let id: i64 = row.get("id");
            let order_id: i64 = row.get("order_id");
            let product_id: i64 = row.get("product_id");

            Ok(OrderItem {
                id,
                order_id,
                product_id,
            })
        }
    }

    impl FromRow for Product {
        fn from_row(row: Row) -> Result<Self, Error> {
            let id: i64 = row.get("id");
            let name: String = row.get("name");

            Ok(Product { id, name })
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
        let pool = Pool::from_env();
        let mut transaction = pool.transaction().await?;

        transaction
            .client()
            .query("DROP TABLE IF EXISTS users CASCADE", &[])
            .await?;

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
        let pool = Pool::from_env();
        let mut transaction = pool.transaction().await?;

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
        let pool = Pool::from_env();

        let mut transaction = pool.transaction().await?;

        transaction
            .client()
            .execute("DROP TABLE IF EXISTS users CASCADE", &[])
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
            r#"SELECT * FROM "users" WHERE "users"."email" = $1 AND "users"."password" = $2 LIMIT 1; INSERT INTO "users" ("email", "password") VALUES ($1, $2) RETURNING *;"#
        );

        query.fetch(&mut transaction).await?;

        Ok(())
    }

    #[test]
    fn test_unique_by() {
        let query = User::create(&[("email", "test@test.com")])
            .unique_by(&["email"])
            .to_sql();
        assert_eq!(
            query,
            r#"INSERT INTO "users" ("email") VALUES ($1) ON CONFLICT ("email") DO UPDATE SET "email" = EXCLUDED."email" RETURNING *"#
        );

        let query = User::filter("email", "test@test.com")
            .find_or_create()
            .unique_by(&["email"])
            .to_sql();
        assert_eq!(
            query,
            r#"SELECT * FROM "users" WHERE "users"."email" = $1 LIMIT 1; INSERT INTO "users" ("email") VALUES ($1) ON CONFLICT ("email") DO UPDATE SET "email" = EXCLUDED."email" RETURNING *;"#
        );
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
