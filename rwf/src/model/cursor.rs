use super::pool::Transaction;
use super::{Column, ToConnectionRequest, Value};
use super::{FromRow, Query, Row};
use crate::config::get_config;
use crate::model::{ConnectionGuard, Error};
use crate::{
    model::{Escape, Placeholders},
    prelude::*,
};
use async_stream::try_stream;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::{marker::PhantomData, sync::atomic::AtomicI64, time::Instant};
use tokio_stream::Stream;
use tracing::{error, info, warn};

/// An Enum declaring the direction the cursor will fetch (important if the cursor is scrollable) #
/// and how far the cursor moves. Not Required to interact directly with
#[derive(
    Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Hash, Default,
)]
pub enum FetchDirection {
    #[default]
    NEXT,
    PRIOR,
    FIRST,
    LAST,
    ABSOLUTE(i64),
    RELATIVE(i64),
    FORWARD(i64),
    ForwardAll,
    BACKWARD(i64),
    BackwardAll,
}

impl std::fmt::Display for FetchDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NEXT => write!(f, "NEXT"),
            Self::PRIOR => write!(f, "PRIOR"),
            Self::FIRST => write!(f, "FIRST"),
            Self::LAST => write!(f, "LAST"),
            Self::ABSOLUTE(n) => write!(f, "ABSOLUTE {}", n),
            Self::RELATIVE(n) => write!(f, "RELATIVE {}", n),
            Self::FORWARD(n) => write!(f, "FORWARD {}", n),
            Self::ForwardAll => write!(f, "FORWARD ALL"),
            Self::BACKWARD(n) => write!(f, "BACKWARD {}", n),
            Self::BackwardAll => write!(f, "BACKWARD ALL"),
        }
    }
}

impl ToSql for FetchDirection {
    fn to_sql(&self) -> String {
        format!(" {} FROM ", self)
    }
}
impl FetchDirection {
    pub fn expected_row_count(&self, row_count: &i64) -> bool {
        use FetchDirection::*;
        match self {
            ABSOLUTE(0) => 0.eq(row_count),
            NEXT | PRIOR | FIRST | LAST => 1.eq(row_count),
            RELATIVE(_) | ABSOLUTE(_) => 1.eq(row_count),

            FORWARD(0) | BACKWARD(0) => 1.eq(row_count),
            FORWARD(n) | BACKWARD(n) => n.abs().eq(row_count),
            ForwardAll | BackwardAll => 0.lt(row_count),
        }
    }
    pub fn mormalized(&self) -> Self {
        use FetchDirection::*;
        match self {
            FORWARD(n) | RELATIVE(n) => {
                if 0.lt(n) {
                    FORWARD(*n)
                } else {
                    BACKWARD(*n)
                }
            }
            BACKWARD(n) => {
                if 0.lt(n) {
                    BACKWARD(*n)
                } else {
                    FORWARD(*n)
                }
            }
            fd => *fd,
        }
    }
    pub fn to_position_update(&self, row_count: &i64) -> Self {
        use FetchDirection::*;
        if self.expected_row_count(row_count) {
            match self {
                ForwardAll => FORWARD(*row_count),
                BackwardAll => BACKWARD(*row_count),
                fd => *fd,
            }
        } else {
            match self {
                NEXT | FIRST | LAST | PRIOR | ForwardAll | BackwardAll => RELATIVE(0),
                ABSOLUTE(_) => ForwardAll,
                RELATIVE(n) => {
                    if 0.eq(n) {
                        RELATIVE(0)
                    } else if 0.lt(n) {
                        ForwardAll
                    } else {
                        BackwardAll
                    }
                }
                FORWARD(n) => {
                    if 0.le(n) {
                        FORWARD(*row_count)
                    } else {
                        FORWARD(-*row_count)
                    }
                }
                BACKWARD(n) => {
                    if 0.le(n) {
                        BACKWARD(*row_count)
                    } else {
                        BACKWARD(-*row_count)
                    }
                }
            }
        }
    }
}

/// An Enum declaring whether the cursor shall fetch entries or just move its position.
/// Not Required to interact directly with.
#[derive(
    Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Default,
)]
pub enum FetchCmd {
    Move,
    #[default]
    Fetch,
}

impl std::fmt::Display for FetchCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FetchCmd::Move => write!(f, "MOVE"),
            FetchCmd::Fetch => write!(f, "FETCH"),
        }
    }
}

impl ToSql for FetchCmd {
    fn to_sql(&self) -> String {
        format!("{}", self)
    }
}

/// A Stmt to interact with the cursor. No need to construrct it directl, this is done by
/// the `[model::cursor::Cursor]` trait. Functions to adjust the `FetchdDirection` and `FetchStmt`
/// are implemented by the Builder Pattern
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Default)]
pub struct FetchStmt {
    cmd: FetchCmd,
    direction: FetchDirection,
    cursor: String,
}

impl ToSql for FetchStmt {
    fn to_sql(&self) -> String {
        format!(
            r#"{}{}"{}""#,
            self.cmd.to_sql(),
            self.direction.to_sql(),
            self.cursor.escape()
        )
    }
}

impl FetchStmt {
    /// Construct a FetchStmt by the cursor name. Usually done by calling `[model::cursor::Cursor::fetch_stmt]`
    /// Inits the `etchStmt` and `FetchDirection` with default values
    /// # Example
    /// ```
    /// use rwf::model::cursor::FetchStmt;
    /// use rwf::model::ToSql;
    /// let stmt = FetchStmt::new("cursor");
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "FETCH NEXT FROM \"cursor\""
    /// );
    /// ```
    pub fn new(cursor: impl ToString) -> Self {
        Self::default().cursor(cursor)
    }
    /// Replace the cursor name
    /// # Example
    /// ```
    /// use rwf::model::cursor::FetchStmt;
    /// use rwf::model::ToSql;
    /// let stmt = FetchStmt::new("cur").cursor("cursor");
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "FETCH NEXT FROM \"cursor\""
    /// );
    /// ```
    pub fn cursor(mut self, cursor: impl ToString) -> Self {
        self.cursor = cursor.to_string();
        self
    }
    /// Change the `FetchCmd` to `[model::cursor::FetchCmd::Move]`
    /// # Example
    /// ```
    /// use rwf::model::cursor::FetchStmt;
    /// use rwf::model::ToSql;
    /// let stmt = FetchStmt::new("cursor").move_cursor();
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "MOVE NEXT FROM \"cursor\""
    /// );
    /// ```
    pub fn move_cursor(mut self) -> Self {
        self.cmd = FetchCmd::Move;
        self
    }
    /// Change the `FetchCmd` to `[model::cursor::FetchCmd::Fetch]`
    /// # Example
    /// ```
    /// use rwf::model::cursor::FetchStmt;
    /// use rwf::model::ToSql;
    /// let stmt = FetchStmt::new("cursor").move_cursor().fetch_cursor();
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "FETCH NEXT FROM \"cursor\""
    /// );
    /// ```
    pub fn fetch_cursor(mut self) -> Self {
        self.cmd = FetchCmd::Fetch;
        self
    }
    /// Change the `FetchDirection` to `[model::cursor::FetchDirection::NEXT]`
    /// # Example
    /// ```
    /// use rwf::model::cursor::FetchStmt;
    /// use rwf::model::ToSql;
    /// let stmt = FetchStmt::new("cursor").next();
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "FETCH NEXT FROM \"cursor\""
    /// );
    /// ```
    pub fn next(mut self) -> Self {
        self.direction = FetchDirection::NEXT;
        self
    }
    /// Change the `FetchDirection` to `[model::cursor::FetchDirection::PRIOR]`
    /// # Example
    /// ```
    /// use rwf::model::cursor::FetchStmt;
    /// use rwf::model::ToSql;
    /// let stmt = FetchStmt::new("cursor").prior();
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "FETCH PRIOR FROM \"cursor\""
    /// );
    /// ```
    pub fn prior(mut self) -> Self {
        self.direction = FetchDirection::PRIOR;
        self
    }
    /// Change the `FetchDirection` to `[model::cursor::FetchDirection::FIRST]`
    /// # Example
    /// ```
    /// use rwf::model::cursor::FetchStmt;
    /// use rwf::model::ToSql;
    /// let stmt = FetchStmt::new("cursor").first();
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "FETCH FIRST FROM \"cursor\""
    /// );
    /// ```
    pub fn first(mut self) -> Self {
        self.direction = FetchDirection::FIRST;
        self
    }
    /// Change the `FetchDirection` to `[model::cursor::FetchDirection::LAST]`
    /// # Example
    /// ```
    /// use rwf::model::cursor::FetchStmt;
    /// use rwf::model::ToSql;
    /// let stmt = FetchStmt::new("cursor").last();
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "FETCH LAST FROM \"cursor\""
    /// );
    /// ```
    pub fn last(mut self) -> Self {
        self.direction = FetchDirection::LAST;
        self
    }
    /// Change the `FetchDirection` to `[model::cursor::FetchDirection::RELATIVE(0)]`
    /// So far the Cursor is Scrollable the last fetched Value will be fetched again
    /// # Example
    /// ```
    /// use rwf::model::cursor::FetchStmt;
    /// use rwf::model::ToSql;
    /// let stmt = FetchStmt::new("cursor").relative(0);
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "FETCH RELATIVE 0 FROM \"cursor\""
    /// );
    /// ```
    pub fn again(self) -> Self {
        self.relative(0)
    }
    /// Change the `FetchDirection` to `[model::cursor::FetchDirection::ABSOLUTE]`
    /// # Example
    /// ```
    /// use rwf::model::cursor::FetchStmt;
    /// use rwf::model::ToSql;
    /// let stmt = FetchStmt::new("cursor").absolute(1);
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "FETCH ABSOLUTE 1 FROM \"cursor\""
    /// );
    /// ```
    pub fn absolute(mut self, n: i64) -> Self {
        self.direction = FetchDirection::ABSOLUTE(n);
        self
    }
    /// Change the `FetchDirection` to `[model::cursor::FetchDirection::RELATIVE]`
    /// # Example
    /// ```
    /// use rwf::model::cursor::FetchStmt;
    /// use rwf::model::ToSql;
    /// let stmt = FetchStmt::new("cursor").relative(1);
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "FETCH RELATIVE 1 FROM \"cursor\""
    /// );
    /// ```
    pub fn relative(mut self, n: i64) -> Self {
        self.direction = FetchDirection::RELATIVE(n);
        self
    }
    /// Change the `FetchDirection` to `[model::cursor::FetchDirection::FORWARD]`
    /// # Example
    /// ```
    /// use rwf::model::cursor::FetchStmt;
    /// use rwf::model::ToSql;
    /// let stmt = FetchStmt::new("cursor").forward(1);
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "FETCH FORWARD 1 FROM \"cursor\""
    /// );
    /// ```
    pub fn forward(mut self, n: i64) -> Self {
        self.direction = FetchDirection::FORWARD(n);
        self
    }
    /// Change the `FetchDirection` to `[model::cursor::FetchDirection::BACKWARD]`
    /// # Example
    /// ```
    /// use rwf::model::cursor::FetchStmt;
    /// use rwf::model::ToSql;
    /// let stmt = FetchStmt::new("cursor").backward(1);
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "FETCH BACKWARD 1 FROM \"cursor\""
    /// );
    /// ```
    pub fn backward(mut self, n: i64) -> Self {
        self.direction = FetchDirection::BACKWARD(n);
        self
    }
    /// Change the `FetchDirection` to `[model::cursor::FetchDirection::ForwardAll]`
    /// # Example
    /// ```
    /// use rwf::model::cursor::FetchStmt;
    /// use rwf::model::ToSql;
    /// let stmt = FetchStmt::new("cursor").forward_all();
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "FETCH FORWARD ALL FROM \"cursor\""
    /// );
    /// ```
    pub fn forward_all(mut self) -> Self {
        self.direction = FetchDirection::ForwardAll;
        self
    }
    /// Change the `FetchDirection` to `[model::cursor::FetchDirection::BackwardAll]`
    /// # Example
    /// ```
    /// use rwf::model::cursor::FetchStmt;
    /// use rwf::model::ToSql;
    /// let stmt = FetchStmt::new("cursor").backward_all();
    /// assert_eq!(
    ///     stmt.to_sql(),
    ///     "FETCH BACKWARD ALL FROM \"cursor\""
    /// );
    /// ```
    pub fn backward_all(mut self) -> Self {
        self.direction = FetchDirection::BackwardAll;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Default)]
pub enum Sensitivity {
    #[default]
    INSENSITIVE,
    ASENSITIVE,
    SENSITIVE,
}

impl std::fmt::Display for Sensitivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::INSENSITIVE => write!(f, "INSENSITIVE"),
            Self::ASENSITIVE => write!(f, "ASENSITIVE"),
            Self::SENSITIVE => write!(f, "SENSITIVE"),
        }
    }
}

impl ToSql for Sensitivity {
    fn to_sql(&self) -> String {
        match self {
            Self::SENSITIVE => unimplemented!("Postgres has no support for SENSITIVE Cursors. SENSITIVE is only implemented for compability with the SQL Standard"),
            sensitivity => format!(" {} ", sensitivity)
        }
    }
}

/// Constructor for a Cursor from a Query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeclareCursor<T: FromRow + ?Sized = Row> {
    query: Query<T>,
    sensitivity: Sensitivity,
    hold: bool,
    scroll: bool,
    name: String,
}

impl<T: FromRow + ?Sized> From<Query<T>> for DeclareCursor<T> {
    fn from(value: Query<T>) -> Self {
        match value {
            Query::Select(select) => {
                let sensitivity = if select.lock.is_locked() {
                    Sensitivity::ASENSITIVE
                } else {
                    Sensitivity::INSENSITIVE
                };
                let query = Query::Select(select);
                DeclareCursor {
                    query,
                    sensitivity,
                    hold: false,
                    scroll: false,
                    name: String::new(),
                }
            }
            Query::Picked(picked) => {
                let sensitivity = if picked.select.lock.is_locked() {
                    Sensitivity::ASENSITIVE
                } else {
                    Sensitivity::INSENSITIVE
                };
                let query = Query::Picked(picked);
                DeclareCursor {
                    query,
                    sensitivity,
                    hold: false,
                    scroll: false,
                    name: String::new(),
                }
            }
            Query::Raw {
                query,
                placeholders,
            } => DeclareCursor {
                query: Query::Raw {
                    query,
                    placeholders,
                },
                sensitivity: Sensitivity::INSENSITIVE,
                hold: false,
                scroll: false,
                name: String::new(),
            },
            _query => unimplemented!("Cursor are only defined for SELECT Queries"),
        }
    }
}

impl<T: Model + Send + Sync> DeclareCursor<T> {
    /// Sets the cursor to Insensitive. This is the Default unless the Query contains FOR UPDATE.
    /// Cursors created with `Sensitivity::INSENSITIVE` will not see changes to the underlying data.
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// use rwf::model::cursor::DeclareCursor;
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String
    /// }
    /// // It's advised to order cursors to make them reproducible
    /// let query = User::all().order(("id", "asc"));
    /// let declare = DeclareCursor::from(query);
    /// // Set the name the cursor will have, otherwise construction will fail.
    /// let declare = declare.name("cursor");
    /// // Explicit set the Cursor to INSENSITIVE
    /// let declare = declare.insensitive();
    /// assert_eq!(
    ///     declare.to_sql(),
    ///     r#"DECLARE "cursor" BINARY INSENSITIVE NO SCROLL CURSOR WITHOUT HOLD FOR SELECT * FROM "users" ORDER BY "id" ASC"#
    /// )
    /// ```
    pub fn insensitive(mut self) -> Self {
        self.sensitivity = Sensitivity::INSENSITIVE;
        self
    }
    /// Sets the cursor to Asensitive. This is the Default if the Query contains FOR UPDATE.
    /// Cursors created with `Sensitivity::ASENSITIVE` will see changes to the underlying data.
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// use rwf::model::cursor::DeclareCursor;
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String
    /// }
    /// // It's advised to order cursors to make them reproducible
    /// let query = User::all().order(("id", "asc"));
    /// let declare = DeclareCursor::from(query);
    /// // Set the name the cursor will have, otherwise construction will fail.
    /// let declare = declare.name("cursor");
    /// // Explicit set the Cursor to ASENSITIVE
    /// let declare = declare.asensitive();
    /// assert_eq!(
    ///     declare.to_sql(),
    ///     r#"DECLARE "cursor" BINARY ASENSITIVE NO SCROLL CURSOR WITHOUT HOLD FOR SELECT * FROM "users" ORDER BY "id" ASC"#
    /// )
    /// ```
    pub fn asensitive(mut self) -> Self {
        self.sensitivity = Sensitivity::ASENSITIVE;
        self
    }
    /// Toggle if the cursor shall be declared with or without hold. Default is without.
    /// Cursors declared `WITH HOLD` can outlive the transaction they were created within or can
    /// even be created outside a transaction.
    /// # CAUTION
    /// If you declare a cursor `WITH HOLD` then you will be responsible to close the cursor once you're done!
    /// Use the statement provided by `[model::cursor::Cursor::close_stmt]` to do this
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// use rwf::model::cursor::DeclareCursor;
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String
    /// }
    /// // It's advised to order cursors to make them reproducible
    /// let query = User::all().order(("id", "asc"));
    /// let declare = DeclareCursor::from(query);
    /// // Set the name the cursor will have, otherwise construction will fail.
    /// let declare = declare.name("cursor");
    /// // Toggle hold
    /// let declare = declare.hold();
    /// assert_eq!(
    ///     declare.to_sql(),
    ///     r#"DECLARE "cursor" BINARY INSENSITIVE NO SCROLL CURSOR WITH HOLD FOR SELECT * FROM "users" ORDER BY "id" ASC"#
    /// )
    /// ```
    pub fn hold(mut self) -> Self {
        self.hold = !self.hold;
        self
    }
    /// Toggle if the cursor shall be declared as scrollable. Scrollable cursors allows to fetch in arbitrary direction, refetch values again and even to restart the cursor.
    /// # CAUTION
    /// If you declare a cursor with `SCROLL` then the Database needs to allocate much more resources to fulfill all actions (at least for complex queries)
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// use rwf::model::cursor::DeclareCursor;
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String
    /// }
    /// // It's advised to order cursors to make them reproducible
    /// let query = User::all().order(("id", "asc"));
    /// let declare = DeclareCursor::from(query);
    /// // Set the name the cursor will have, otherwise construction will fail.
    /// let declare = declare.name("cursor");
    /// // Toggle scrollable
    /// let declare = declare.scroll();
    /// assert_eq!(
    ///     declare.to_sql(),
    ///     r#"DECLARE "cursor" BINARY INSENSITIVE SCROLL CURSOR WITHOUT HOLD FOR SELECT * FROM "users" ORDER BY "id" ASC"#
    /// )
    /// ```
    pub fn scroll(mut self) -> Self {
        self.scroll = !self.scroll;
        self
    }
    /// Set the name of the cursor. Without a name executing the Statement will result in an Error.
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// use rwf::model::cursor::DeclareCursor;
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String
    /// }
    /// // It's advised to order cursors to make them reproducible
    /// let query = User::all().order(("id", "asc"));
    /// let declare = DeclareCursor::from(query);
    /// // Set the name the cursor will have, otherwise construction will fail.
    /// let declare = declare.name("cursor");
    /// assert_eq!(
    ///     declare.to_sql(),
    ///     r#"DECLARE "cursor" BINARY INSENSITIVE NO SCROLL CURSOR WITHOUT HOLD FOR SELECT * FROM "users" ORDER BY "id" ASC"#
    /// )
    /// ```
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
    /// Get a reference to the `model::placeholders::Placeholders` of the query.
    /// Indented for internal use only
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// use rwf::model::cursor::DeclareCursor;
    /// use rwf::model::Placeholders;
    ///
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String
    /// }
    /// // It's advised to order cursors to make them reproducible
    /// let query = User::all().order(("id", "asc")).filter_gt("id", 5);
    /// let declare = DeclareCursor::from(query);
    /// assert_eq!(
    ///     declare.placeholders(),
    ///     &Placeholders::from(vec![Value::Int(5)])
    /// )
    /// ```
    pub fn placeholders(&self) -> &Placeholders {
        // Handle Raw Querys as model::select::FilterQuery is not implemented for them
        if let Query::Raw {
            query: _query,
            placeholders,
        } = &self.query
        {
            placeholders
        } else {
            self.query.get_placeholders()
        }
    }
    /// Construct a `model::cursor::ModelCursor` from the declaration.
    /// # CAUTION
    /// If the query is `Query::Picked` or `Query::Raw`, then you are responsible that the Parameter `T`s `[model::FromRow]` implementation is able to create an Object of type `T`
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// use rwf::model::cursor::{DeclareCursor, ModelCursor, Cursor};
    ///
    /// use rwf::model::Pool;
    ///
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String
    /// }
    /// #[tokio::main]
    /// async fn main() -> Result<(), Error> {
    /// // Use a transaction, otherwise the cursor have to be declared WITH HOLD
    /// let mut tx = Pool::pool().transaction().await?;
    /// tx.query_cached(format!(r#"CREATE TABLE IF NOT EXISTS users(id bigserial primary key, name text)"#).as_str(), &[]).await?;
    /// let cursor = DeclareCursor::from(
    ///     User::all().order(("id", "asc"))
    /// )
    ///     .name("cursor")
    ///     .create_model_cursor(&mut tx).await?;
    /// assert!(!cursor.scrollable());
    /// assert!(!cursor.with_hold());
    /// assert_eq!(cursor.name(), "cursor");
    /// tx.rollback().await?;
    /// Ok(())
    /// }
    /// ```
    pub async fn create_model_cursor(
        self,
        conn: impl ToConnectionRequest<'_>,
    ) -> Result<ModelCursor<T>, Error> {
        let conn = conn.to_connection_request()?.connection().unwrap();
        conn.query_cached(self.to_sql().as_str(), &[]).await?;
        let cur = ModelCursor::from(self);
        Ok(cur)
    }
    /// Construct a `model::cursor::TxModelCursor` from the declaration.
    /// # CAUTION
    /// If the query is `Query::Picked` or `Query::Raw`, then you are responsible that the Parameter `T`s `[model::FromRow]` implementation is able to create an Object of type `T`
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// use rwf::model::cursor::{DeclareCursor, TxModelCursor, Cursor, TransactionCursor};
    ///
    /// use rwf::model::Pool;
    ///
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String
    /// }
    /// #[tokio::main]
    /// async fn main() -> Result<(), Error> {
    /// // INTERNAL: Ensure Table for Test exists. And give the Transaction to Cursor. Usually this is not required as the Cursor is able to create a Transaction by itself
    /// let mut tx = Pool::pool().transaction().await?;
    /// tx.query_cached(format!(r#"CREATE TABLE IF NOT EXISTS users(id bigserial primary key, name text)"#).as_str(), &[]).await?;
    /// let mut cursor = DeclareCursor::from(
    ///     User::all().order(("id", "asc"))
    /// )
    ///     .name("cursor")
    ///     .create_tx_model_cursor(Some(tx)).await?;
    /// assert!(!cursor.scrollable());
    /// assert!(!cursor.with_hold());
    /// assert_eq!(cursor.name(), "cursor");
    /// // Close the Cursor and take back the Transaction -- Not required but a good practice
    /// let mut tx = cursor.close().await?;
    /// tx.rollback().await?;
    /// Ok(())
    /// }
    /// ```
    pub async fn create_tx_model_cursor(
        self,
        tx: Option<Transaction>,
    ) -> Result<TxModelCursor<T>, Error> {
        let mut tx = tx.unwrap_or(Pool::pool().transaction().await?);
        self.create_model_cursor(&mut tx)
            .await
            .map(|mc| TxModelCursor { inner: mc, tx })
    }
    /// Construct a `model::cursor::SelectiveCursor` from the declaration.
    /// # CAUTION
    /// Only defined for `Query::Picked`
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// use rwf::model::cursor::{DeclareCursor, SelectiveCursor, Cursor};
    ///
    /// use rwf::model::Pool;
    ///
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String
    /// }
    /// #[tokio::main]
    /// async fn main() -> Result<(), Error> {
    /// // Use a transaction, otherwise the cursor have to be declared WITH HOLD
    /// let mut tx = Pool::pool().transaction().await?;
    /// tx.query_cached(format!(r#"CREATE TABLE IF NOT EXISTS users(id bigserial primary key, name text)"#).as_str(), &[]).await?;
    /// let cursor = DeclareCursor::from(
    ///     User::all().order(("id", "asc")).select_columns(&["name"])
    /// )
    ///     .name("cursor")
    ///     .create_selective_cursor(&mut tx).await?;
    /// assert!(!cursor.scrollable());
    /// assert!(!cursor.with_hold());
    /// assert_eq!(cursor.name(), "cursor");
    /// tx.rollback().await?;
    /// Ok(())
    /// }
    /// ```
    pub async fn create_selective_cursor(
        self,
        conn: impl ToConnectionRequest<'_>,
    ) -> Result<SelectiveCursor, Error> {
        match &self.query {
            Query::Picked(_) => {
                let request = conn.to_connection_request()?;
                match request.get().await? {
                    None => {
                        let conn = request.connection().unwrap();
                        conn.query_cached(self.to_sql().as_str(), &[]).await?;
                        Ok(SelectiveCursor::from(self))
                    }
                    Some(mut guard) => {
                        guard.query_cached(self.to_sql().as_str(), &[]).await?;
                        Ok(SelectiveCursor::from(self))
                    }
                }
            }
            _query => Err(Error::QueryError(
                "Expected a picked query.".to_string(),
                _query.to_sql(),
            )),
        }
    }
    /// Construct a `model::cursor::TxSelectiveCursor` from the declaration.
    /// # CAUTION
    /// This is only defined for `Query::Picked`
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// use rwf::model::cursor::{DeclareCursor, TxSelectiveCursor, Cursor, TransactionCursor};
    ///
    /// use rwf::model::Pool;
    ///
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String
    /// }
    /// #[tokio::main]
    /// async fn main() -> Result<(), Error> {
    /// // INTERNAL: Ensure Table for Test exists. And give the Transaction to Cursor. Usually this is not required as the Cursor is able to create a Transaction by itself
    /// let mut tx = Pool::pool().transaction().await?;
    /// tx.query_cached(format!(r#"CREATE TABLE IF NOT EXISTS users(id bigserial primary key, name text)"#).as_str(), &[]).await?;
    /// let declare = DeclareCursor::from(
    ///     User::all().order(("id", "asc")).select_columns(&["name"])
    /// ).name("cursor");
    /// assert_eq!(
    ///     declare.to_sql(),
    ///     r#"DECLARE "cursor" BINARY INSENSITIVE NO SCROLL CURSOR WITHOUT HOLD FOR SELECT "users"."name" FROM "users" ORDER BY "id" ASC"#
    /// );
    /// let mut cursor = declare.create_tx_selective_cursor(Some(tx)).await?;
    ///
    /// assert!(!cursor.scrollable());
    /// assert!(!cursor.with_hold());
    /// assert_eq!(cursor.name(), "cursor");
    /// // Close the Cursor and take back the Transaction -- Not required but a good practice
    /// let mut tx = cursor.close().await?;
    /// tx.rollback().await?;
    /// Ok(())
    /// }
    /// ```
    pub async fn create_tx_selective_cursor(
        self,
        tx: Option<Transaction>,
    ) -> Result<TxSelectiveCursor, Error> {
        let mut tx = tx.unwrap_or(Pool::pool().transaction().await?);
        let cur = self.create_selective_cursor(&mut tx).await?;
        Ok(TxSelectiveCursor { inner: cur, tx })
    }
}

impl<T: Model> ToSql for DeclareCursor<T> {
    fn to_sql(&self) -> String {
        format!(
            r#"DECLARE "{}" BINARY{}{}SCROLL CURSOR {} HOLD FOR {}"#,
            self.name.escape(),
            self.sensitivity.to_sql(),
            if self.scroll { "" } else { "NO " },
            if self.hold { "WITH" } else { "WITHOUT" },
            self.query.to_sql()
        )
    }
}

/// Cursor MetaData as defined in the `DeclareCursor` Statement (+ the time of creation)
/// In fact, this is the only required struct to work with Cursors.
/// Accessed is the struct only by the `Cursor` trait implemented for `CursorData`
#[derive(Debug, Clone, Ord, PartialEq, PartialOrd, Eq, Serialize, Deserialize)]
pub struct CursorMeta {
    sensitivity: Sensitivity,
    is_scroll_able: bool,
    hold: bool,
    #[serde(skip, default = "std::time::Instant::now")]
    created: Instant,
    name: String,
}
impl<T> From<DeclareCursor<T>> for CursorMeta
where
    T: FromRow,
{
    fn from(value: DeclareCursor<T>) -> Self {
        Self {
            sensitivity: value.sensitivity,
            is_scroll_able: value.scroll,
            hold: value.scroll,
            created: Instant::now(),
            name: value.name,
        }
    }
}

/// Extension of the CursorMeta. Not Really Required, but provides cheap information about the cursor
/// As such the position and the number of fetched entries
/// Implements the `[model::cursor::Cursor]` where all other Extensions Depends on and `Deref` to
#[derive(Debug, Serialize, Deserialize)]
pub struct CursorData {
    meta: CursorMeta,
    #[serde(skip, default = "std::time::Instant::now")]
    used: Instant,
    position: AtomicI64,
    fetched: AtomicI64,
}
// Just some standard implementations which were not derived because of AtomicI64
// Properly never used, but implemented just in case they become required
impl Clone for CursorData {
    fn clone(&self) -> Self {
        Self {
            meta: self.meta.clone(),
            used: self.used,
            position: AtomicI64::new(self.position.load(std::sync::atomic::Ordering::Relaxed)),
            fetched: AtomicI64::new(self.fetched.load(std::sync::atomic::Ordering::Relaxed)),
        }
    }
}
impl PartialEq for CursorData {
    fn eq(&self, other: &Self) -> bool {
        self.meta().eq(other.meta())
            && self.position().eq(&other.position())
            && self.fetched().eq(&other.fetched())
            && self.last_used().eq(&other.last_used())
    }
}
impl Eq for CursorData {}
impl Ord for CursorData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.meta().cmp(other.meta()) {
            std::cmp::Ordering::Equal => match self.position().cmp(&other.position()) {
                std::cmp::Ordering::Equal => match self.fetched().cmp(&other.fetched()) {
                    std::cmp::Ordering::Equal => self.last_used().cmp(&other.last_used()),
                    ord => ord,
                },
                ord => ord,
            },
            ord => ord,
        }
    }
}
impl PartialOrd for CursorData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> From<DeclareCursor<T>> for CursorData
where
    T: FromRow,
{
    fn from(value: DeclareCursor<T>) -> Self {
        Self {
            meta: CursorMeta::from(value),
            used: Instant::now(),
            position: AtomicI64::new(0),
            fetched: AtomicI64::new(0),
        }
    }
}
impl Cursor for CursorData {
    fn meta(&self) -> &CursorMeta {
        &self.meta
    }
    fn last_used(&self) -> Instant {
        self.used
    }
    fn position(&self) -> i64 {
        self.position.load(std::sync::atomic::Ordering::Relaxed)
    }
    fn fetched(&self) -> i64 {
        self.fetched.load(std::sync::atomic::Ordering::Relaxed)
    }
    fn update_used(&mut self) {
        self.used = Instant::now();
    }
    fn get_position_mut(&mut self) -> &mut AtomicI64 {
        &mut self.position
    }
    fn get_fetched_mut(&mut self) -> &mut AtomicI64 {
        &mut self.fetched
    }
}
#[derive(Debug)]
pub struct ModelCursor<T>
where
    T: Model + Send + Sync,
    Self: Send + Sync,
{
    inner: CursorData,
    _marker: PhantomData<T>,
}

impl<T> From<DeclareCursor<T>> for ModelCursor<T>
where
    T: Model + Send + Sync,
{
    fn from(value: DeclareCursor<T>) -> Self {
        Self {
            inner: CursorData::from(value),
            _marker: PhantomData,
        }
    }
}
impl<T> Deref for ModelCursor<T>
where
    T: Model + Send + Sync,
{
    type Target = dyn Cursor;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T> DerefMut for ModelCursor<T>
where
    T: Model + Send + Sync,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[derive(Debug)]
pub struct SelectiveCursor
where
    Self: Send + Sync,
{
    inner: CursorData,
    columns: Vec<Column>,
}

impl<T> From<DeclareCursor<T>> for SelectiveCursor
where
    T: FromRow,
{
    fn from(value: DeclareCursor<T>) -> Self {
        let columns = match value.query {
            Query::Picked(ref picked) => picked.clone().columns(),
            _query => unimplemented!(),
        };
        Self {
            inner: CursorData::from(value),
            columns,
        }
    }
}
impl Deref for SelectiveCursor {
    type Target = dyn Cursor;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for SelectiveCursor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct TxModelCursor<T>
where
    T: Model + Send + Sync,
    Self: Send + Sync,
{
    inner: ModelCursor<T>,
    tx: Transaction,
}

impl<T> Deref for TxModelCursor<T>
where
    T: Model + Send + Sync + 'static,
{
    type Target = dyn Cursor;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}
impl<T> DerefMut for TxModelCursor<T>
where
    T: Model + Send + Sync + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

impl<T> TransactionCursor for TxModelCursor<T>
where
    T: Model + Send + Sync + 'static,
{
    fn tx(&mut self) -> &mut Transaction {
        &mut self.tx
    }
    fn take_tx(self) -> Transaction {
        self.tx
    }
}

pub struct TxSelectiveCursor
where
    Self: Send + Sync,
{
    inner: SelectiveCursor,
    tx: Transaction,
}
impl Deref for TxSelectiveCursor {
    type Target = dyn Cursor;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}
impl DerefMut for TxSelectiveCursor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}
impl AsRef<Vec<Column>> for TxSelectiveCursor {
    fn as_ref(&self) -> &Vec<Column> {
        &self.inner.columns
    }
}

impl TransactionCursor for TxSelectiveCursor {
    fn tx(&mut self) -> &mut Transaction {
        &mut self.tx
    }
    fn take_tx(self) -> Transaction {
        self.tx
    }
}

/// The main Trait to interact with a Cursor
pub trait Cursor: Sync + Send {
    /// Getter for the Cursors Core Data, like name, sensitivity etc
    fn meta(&self) -> &CursorMeta;
    /// Create a `FetchStmt` with the name of the Cursor set.
    fn fetch_stmt(&self) -> FetchStmt {
        FetchStmt::new(self.name())
    }
    /// Construct the Statement required to Close the Cursor
    fn close_stmt(&self) -> String {
        format!(r#"CLOSE "{}""#, self.name().escape())
    }
    /// Indicates whether the Cursor is Scrollable (Constructed with `SCROLL`)
    fn scrollable(&self) -> bool {
        self.meta().is_scroll_able
    }
    /// Indicates whether the Cursor can outlive the `Transaction` it was created by (Constructed with `WITH HOLD`)
    fn with_hold(&self) -> bool {
        self.meta().hold
    }
    /// Indicates whether the Cursor is `Sensitivity::INSENSITIVE` (Constructed with `INSENSITIVE`)
    fn insensitive(&self) -> bool {
        Sensitivity::INSENSITIVE.eq(&self.meta().sensitivity)
    }
    /// Indicates whether the Cursor is `Sensitivity::ASENSITIVE` (Constructed with `ASENSITIVE`)
    fn asensitive(&self) -> bool {
        Sensitivity::ASENSITIVE.eq(&self.meta().sensitivity)
    }
    /// The name used to create the Cursor
    fn name(&self) -> &str {
        self.meta().name.as_str()
    }
    /// The creation time of the Server
    fn created(&self) -> Instant {
        self.meta().created
    }
    /// Last time the Cursor was fetched
    fn last_used(&self) -> Instant;
    /// The Cursors current position. If only `FetchDirection::NEXT` was used, then this should match `[model::cursor::Cursor::fetched]`
    fn position(&self) -> i64;
    /// The number of fetched Records by the Server
    fn fetched(&self) -> i64;
    /// Updates the Cursor Stats. Warps `update_position` `update_used` and `update_fetched`
    fn update(&mut self, fd: FetchDirection, row_count: i64) {
        self.update_used();
        self.update_fetched(row_count);
        self.update_position(fd.to_position_update(&row_count));
    }
    /// Adjust the current Position of the Cursor
    fn update_position(&mut self, fd: FetchDirection) {
        use FetchDirection::*;
        let order = std::sync::atomic::Ordering::Relaxed;
        let cur = self.get_position_mut();
        match fd {
            NEXT => cur.fetch_add(1, order),
            PRIOR => cur.fetch_add(-1, order),
            FIRST => cur.swap(1, order),
            LAST => cur.swap(-1, order),
            ABSOLUTE(n) => cur.swap(n, order),
            RELATIVE(n) => cur.fetch_add(n, order),
            FORWARD(n) => cur.fetch_add(n, order),
            BACKWARD(n) => cur.fetch_add(-n, order),
            ForwardAll => cur.swap(i64::MAX, order),
            BackwardAll => cur.swap(0, order),
        };
    }
    /// Increase the number of fetched records
    fn update_fetched(&mut self, count: i64) {
        self.get_fetched_mut()
            .fetch_add(count, std::sync::atomic::Ordering::Relaxed);
    }
    /// Update the last used time
    fn update_used(&mut self) -> ();
    /// Get the raw position
    fn get_position_mut(&mut self) -> &mut AtomicI64;
    /// Get the raw fetched counter
    fn get_fetched_mut(&mut self) -> &mut AtomicI64;
}
#[async_trait]
pub trait TransactionCursor
where
    Self: Send + Sync + Deref<Target = dyn Cursor> + DerefMut<Target = dyn Cursor> + 'static,
{
    /// Getter for the Transaction hold by the Cursor
    fn tx(&mut self) -> &mut Transaction;
    /// Destroy the Cursor and return the `Transaction` hold by.
    /// Should not be called directly, as even an Cursor created WITHOUT HOLD will stay in the Database Memory till the Transaction is closed.
    /// Is called by `Self::close`
    fn take_tx(self) -> Transaction;
    /// Close the Cursor and return the Transaction
    async fn close(mut self) -> Result<Transaction, Error>
    where
        Self: Sized,
    {
        let close_stmt = self.close_stmt();
        self.tx().query_cached(close_stmt.as_str(), &[]).await?;
        info!(
            "Closed Cursor {} after ({:.3} ms) at position {}. Fetched Rows {}",
            self.name(),
            self.created().elapsed().as_secs_f64(),
            self.position(),
            self.fetched()
        );
        Ok(self.take_tx())
    }
    /// Create a new Savepoint in the Transaction
    /// Just a shortcut for `[model::pool::transaction::Transaction::savepoint]`
    async fn savepoint(&mut self) -> Result<(), Error> {
        self.tx().savepoint().await
    }
    /// Rollback to the latest savepoint (if one exists)
    /// Just a shortcut for `[model::pool::transaction::Transaction::rollback_savepoint]`
    async fn rollback_savepoint(&mut self) -> Result<(), Error> {
        self.tx().rollback_savepoint().await
    }
    /// Release the latest savepoint (if one exists)
    /// /// Just a shortcut for `[model::pool::transaction::Transaction::release_savepoint]`
    async fn release_savepoint(&mut self) -> Result<(), Error> {
        self.tx().release_savepoint().await
    }
}

/// A Cursor which is self-contained fetchable. Means everything to handle fetches from the Cursor is implemented
#[async_trait]
pub trait FetchableCursor {
    type Output;
    ///  Getter for the Cursor data. Usually this is implemented by `Deref` but in case one want to use a own implementation
    fn cursor_data(&self) -> &dyn Cursor;
    ///  Getter for the mutable Cursor data. Usually this is implemented by `DerefMut` but in case one want to use a own implementation
    fn cursor_data_mut(&mut self) -> &mut dyn Cursor;
    /// Declares how the Cursor get a Connection to execute a `FetchStmt`
    async fn conn(&mut self) -> Result<&mut ConnectionGuard, Error>;
    /// Internal fetch mechanism. If you overwrite this, then you will change the way the Trait work fundamentally
    async fn fetch_internal(&mut self, stmt: FetchStmt) -> Result<Vec<tokio_postgres::Row>, Error> {
        let query = stmt.to_sql();
        if get_config().general.log_queries {
            info!("Execute FetchStmt {}", query);
        }
        match stmt.cmd {
            FetchCmd::Fetch => match self.conn().await?.query_cached(query.as_str(), &[]).await {
                Ok(rows) => {
                    self.cursor_data_mut()
                        .update(stmt.direction, rows.len() as i64);
                    Ok(rows)
                }
                Err(Error::RecordNotFound) => Err(Error::RecordNotFound),
                Err(e) => {
                    if let Error::RecordNotFound = e {
                        Err(Error::RecordNotFound)
                    } else {
                        error!(
                            "Failed to fetch rows from Cursor {} -- Error -> {:?}",
                            self.cursor_data().name(),
                            e
                        );
                        Err(e)
                    }
                }
            },
            FetchCmd::Move => match self.conn().await?.query_cached(query.as_str(), &[]).await {
                Err(e) => {
                    error!(
                        "Failed to MOVE Cursor {} -- Error -> {:?}",
                        self.cursor_data().name(),
                        e
                    );
                    Err(e)
                }
                Ok(mut rows) => {
                    if let Some(row) = rows.pop() {
                        match row.try_get::<usize, i64>(0) {
                            Ok(moved) => {
                                info!("Moved Cursor by {}", moved);
                                self.cursor_data_mut().update_used();
                                self.cursor_data_mut()
                                    .update_position(stmt.direction.to_position_update(&moved));
                                Ok(vec![])
                            }
                            Err(e) => {
                                warn!("Failed to update the current position of the cursor {} -- Error -> {:?}", self.cursor_data().name(), e);
                                Err(Error::DatabaseError(e))
                            }
                        }
                    } else {
                        self.cursor_data_mut().update_used();
                        self.cursor_data_mut().update_position(stmt.direction);
                        Ok(vec![])
                    }
                }
            },
        }
    }
    /// Call to `Self::fetch_internal` and handle the type conversion after.
    async fn fetch(&mut self, stmt: FetchStmt) -> Result<Vec<Self::Output>, Error>;
    /// Same as `Self::fetch` but verifies that the `FetchStmt` will return one result as maximum before.
    /// Internaly calls `Self::fetch`
    /// Will result in `[model::error::Error::QueryError]` if `FetchStmt` contains a `FetchDirection` which could return more then one Result or if the `FetchCmd` is `Move`
    async fn fetch_one(&mut self, stmt: FetchStmt) -> Result<Self::Output, Error> {
        use FetchDirection::*;
        if stmt.cmd == FetchCmd::Move {
            return Err(Error::QueryError("Exprected a FetchStmt with FETCH as the FetchCmd. Unable to fetch from cursor with a MOVE.".to_string(), stmt.to_sql()));
        }
        match &stmt.direction {
            FIRST | LAST | NEXT | PRIOR => {}
            ABSOLUTE(_) | RELATIVE(_) => {}
            BACKWARD(n) | FORWARD(n) => {
                if 1.lt(n) || (-1).gt(n) {
                    return Err(Error::QueryError(
                        "Expected a FetchStmt with direction set to a single fetching one."
                            .to_string(),
                        stmt.to_sql(),
                    ));
                }
            }
            ForwardAll | BackwardAll => {
                return Err(Error::QueryError(
                    "Expected a FetchStmt with direction set to a single fetching one.".to_string(),
                    stmt.to_sql(),
                ));
            }
        };
        Ok(self.fetch(stmt).await?.pop().unwrap())
    }
    async fn fetch_one_optional(&mut self, stmt: FetchStmt) -> Result<Option<Self::Output>, Error> {
        match self.fetch_one(stmt).await {
            Ok(row) => Ok(Some(row)),
            Err(Error::RecordNotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }
    async fn fetch_optional(
        &mut self,
        stmt: FetchStmt,
    ) -> Result<Option<Vec<Self::Output>>, Error> {
        match self.fetch(stmt).await {
            Ok(rows) => {
                if !rows.is_empty() {
                    Ok(Some(rows))
                } else {
                    Ok(None)
                }
            }
            Err(Error::RecordNotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }
    fn stream(&mut self, stmt: FetchStmt) -> impl Stream<Item = Result<Self::Output, Error>>
    where
        Self: Sized + Send + Sync + Unpin,
    {
        Box::pin(try_stream! {
            while let Some(obj) = self.fetch_one_optional(stmt.clone()).await? {
                yield obj;
            }
        })
    }
}

#[async_trait]
impl<T> FetchableCursor for TxModelCursor<T>
where
    T: Model + Send + Sync + 'static,
{
    type Output = T;

    fn cursor_data(&self) -> &dyn Cursor {
        self.deref()
    }

    fn cursor_data_mut(&mut self) -> &mut dyn Cursor {
        self.deref_mut()
    }

    async fn conn(&mut self) -> Result<&mut ConnectionGuard, Error> {
        Ok(self.tx().to_connection_request()?.connection().unwrap())
    }
    async fn fetch(&mut self, stmt: FetchStmt) -> Result<Vec<Self::Output>, Error> {
        let rows = self.fetch_internal(stmt).await?;
        if rows.is_empty() {
            Err(Error::RecordNotFound)
        } else {
            let mut data = Vec::with_capacity(rows.len());
            for row in rows {
                let converted = Self::Output::from_row(row);
                data.push(converted?);
            }
            Ok(data)
        }
    }
}

#[async_trait]
impl FetchableCursor for TxSelectiveCursor {
    type Output = HashMap<Column, Value>;

    fn cursor_data(&self) -> &dyn Cursor {
        self.deref()
    }

    fn cursor_data_mut(&mut self) -> &mut dyn Cursor {
        self.deref_mut()
    }

    async fn conn(&mut self) -> Result<&mut ConnectionGuard, Error> {
        Ok(self.tx().to_connection_request()?.connection().unwrap())
    }
    async fn fetch(&mut self, stmt: FetchStmt) -> Result<Vec<Self::Output>, Error> {
        let rows = self.fetch_internal(stmt).await?;
        if rows.is_empty() {
            Err(Error::RecordNotFound)
        } else {
            let mut data = Vec::with_capacity(rows.len());
            for row in rows {
                let mut map = HashMap::with_capacity(row.len());
                for col in self.as_ref() {
                    map.insert(col.clone(), row.try_get(col.get_name())?);
                }
                data.push(map);
            }
            Ok(data)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::cursor::{DeclareCursor, FetchStmt, FetchableCursor, TransactionCursor};
    use crate::model::prelude::*;
    use crate::model::Placeholders;
    use crate::prelude::*;
    use tokio::time::sleep;
    use tokio_postgres::Row;
    use tokio_stream::StreamExt;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    struct Employee {
        id: Option<i32>,
        name: String,
        boss: Option<i32>,
    }
    impl FromRow for Employee {
        fn from_row(row: Row) -> Result<Self, crate::model::Error>
        where
            Self: Sized,
        {
            Ok(Self {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                boss: row.try_get("boss")?,
            })
        }
    }
    impl Model for Employee {
        fn table_name() -> &'static str {
            "employees"
        }

        fn column_names() -> &'static [&'static str] {
            &["name", "boss"]
        }

        fn id(&self) -> Value {
            self.id.to_value()
        }

        fn values(&self) -> Vec<Value> {
            vec![self.name.to_value(), self.boss.to_value()]
        }

        fn foreign_key() -> &'static str {
            "employee_id"
        }
    }
    impl Employee {
        fn raw() -> Scope<Self> {
            Query::Raw {
                query: "SELECT * FROM (VALUES (1, 'bigboss', NULL), (2, 'section leader', 1), (3, 'team leader', 2), (4, 'Worker One', 3), (5, 'Worker two', 3)) as employees(id, name, boss) ORDER BY id asc".to_string(),
                placeholders: Placeholders::new()
            }
        }
        fn single() -> Scope<Self> {
            Query::Raw {
                query: "SELECT * FROM (VALUES (1, 'bigboss', NULL::INT)) as employees(id, name, boss) ORDER BY id asc".to_string(),
                placeholders: Placeholders::new()
            }
        }
        fn selective() -> Scope<Self> {
            Self::raw()
                .select_with("employed")
                .select_columns(&["id", "name"])
        }
    }
    #[tokio::test]
    async fn test_tx_cursor_create() {
        let res = DeclareCursor::from(Employee::raw())
            .name("cursor")
            .create_tx_model_cursor(None)
            .await;
        assert!(res.is_ok());
        let cursor = res.unwrap();
        assert_eq!(cursor.name(), "cursor");
        assert_eq!(
            cursor.created().elapsed().as_secs(),
            cursor.last_used().elapsed().as_secs()
        );
        assert!(!cursor.scrollable());
        assert!(!cursor.with_hold());
        assert!(cursor.insensitive());
        assert_eq!(cursor.fetched(), 0);
        assert_eq!(cursor.position(), 0);
        assert_eq!(cursor.fetch_stmt(), FetchStmt::new("cursor"));
        cursor.close().await.unwrap().rollback().await.unwrap();
    }
    #[tokio::test]
    async fn test_fetch_cursor() {
        let res = DeclareCursor::from(Employee::raw())
            .name("cursor")
            .create_tx_model_cursor(None)
            .await;
        let mut cursor = res.unwrap();
        let user = cursor.fetch_one(cursor.fetch_stmt()).await;
        assert!(user.is_ok());
        let user = user.unwrap();
        assert_eq!(user.id(), Value::Optional(Box::new(Some(Value::Int(1)))));

        sleep(tokio::time::Duration::from_secs(1)).await;

        let user = cursor.fetch_one(cursor.fetch_stmt()).await;
        assert!(user.is_ok());
        let user = user.unwrap();
        assert_eq!(user.id(), Value::Optional(Box::new(Some(Value::Int(2)))));

        assert_eq!(cursor.position(), 2);
        assert_eq!(cursor.fetched(), 2);
        assert_ne!(
            cursor.last_used().elapsed().as_secs(),
            cursor.created().elapsed().as_secs()
        );
        cursor.close().await.unwrap().rollback().await.unwrap();
    }

    #[tokio::test]
    async fn test_scroll_cursor() {
        let res = DeclareCursor::from(Employee::raw())
            .name("cursor")
            .scroll()
            .create_tx_model_cursor(None)
            .await;
        let mut cursor = res.unwrap();
        let user1 = cursor.fetch_one(cursor.fetch_stmt()).await.unwrap();
        assert_eq!(cursor.fetched(), 1);
        assert_eq!(cursor.position(), 1);
        let user2 = cursor.fetch_one(cursor.fetch_stmt().again()).await.unwrap();
        assert_eq!(user1, user2);
        assert_eq!(cursor.position(), 1);
        assert_eq!(cursor.fetched(), 2);
    }

    #[tokio::test]
    async fn test_move_cursor() {
        let mut cursor = DeclareCursor::from(Employee::raw())
            .name("cursor")
            .scroll()
            .create_tx_model_cursor(None)
            .await
            .unwrap();
        assert!(cursor
            .fetch_one(cursor.fetch_stmt().move_cursor())
            .await
            .is_err());
        assert!(cursor
            .fetch_one_optional(cursor.fetch_stmt().move_cursor())
            .await
            .is_err());
        assert!(cursor
            .fetch(cursor.fetch_stmt().move_cursor())
            .await
            .is_err());
        assert!(cursor
            .fetch_optional(cursor.fetch_stmt().move_cursor())
            .await
            .is_ok());
        assert_eq!(cursor.position(), 2);
        assert_eq!(cursor.fetched(), 0);
        cursor.close().await.unwrap().rollback().await.unwrap();
    }

    #[tokio::test]
    async fn test_end_of_scursor() {
        let mut cursor = DeclareCursor::from(Employee::raw())
            .name("cursor")
            .scroll()
            .create_tx_model_cursor(None)
            .await
            .unwrap();
        assert!(cursor.fetch(cursor.fetch_stmt().forward(4)).await.is_ok());
        let last_user = cursor.fetch_one(cursor.fetch_stmt()).await.unwrap();
        assert_eq!(
            last_user.id(),
            Value::Optional(Box::new(Some(Value::Int(5))))
        );
        assert!(cursor.fetch_one(cursor.fetch_stmt()).await.is_err());
        assert!(cursor.fetch(cursor.fetch_stmt()).await.is_err());
        assert!(cursor.fetch_one_optional(cursor.fetch_stmt()).await.is_ok());
        assert!(cursor.fetch_optional(cursor.fetch_stmt()).await.is_ok());
        cursor.close().await.unwrap().rollback().await.unwrap();
    }

    #[tokio::test]
    async fn test_stream() {
        let mut cursor = DeclareCursor::from(Employee::single())
            .name("cursor")
            .scroll()
            .create_tx_model_cursor(None)
            .await
            .unwrap();
        let mut stream = cursor.stream(cursor.fetch_stmt());
        let user = stream.next().await;
        assert!(user.is_some());
        let user = user.unwrap();
        assert!(user.is_ok());
        let user = user.unwrap();
        assert_eq!(user.id(), Value::Optional(Box::new(Some(Value::Int(1)))));

        let user = stream.next().await;
        assert!(user.is_none());
        let user = stream.next().await;
        assert!(user.is_none());
        drop(stream);
        let tx = cursor.close().await.unwrap();
        tx.close();
        tx.rollback().await.unwrap();

        //assert!(user.is_some());
        //let user = user.unwrap();
        //assert_eq!(user.id(), Value::Optional(Box::new(Some(Value::Int(1)))));
        //let user = cursor.next().await;
        //assert!(user.is_none());
    }

    #[tokio::test]
    async fn test_stream_implementation() {
        let mut cursor = DeclareCursor::from(Employee::raw())
            .name("cursor")
            .scroll()
            .create_tx_model_cursor(None)
            .await
            .unwrap();

        let users: Vec<Employee> = cursor
            .stream(cursor.fetch_stmt())
            .map(|res| res.unwrap())
            .collect()
            .await;
        assert_eq!(users.len(), 5);
        assert_eq!(cursor.fetched(), 5);
        assert_eq!(cursor.position(), 5);

        let mut employees: Vec<Employee> = cursor
            .stream(cursor.fetch_stmt().prior())
            .map(|res| res.unwrap())
            .collect()
            .await;
        assert_eq!(employees.len(), 5);
        assert_eq!(cursor.fetched(), 10);
        assert_eq!(cursor.position(), 0);

        assert_ne!(employees, users);
        employees.reverse();
        assert_eq!(employees, users);
        cursor.close().await.unwrap().rollback().await.unwrap();
    }

    #[tokio::test]
    async fn test_invalid_args() {
        let mut cursor = DeclareCursor::from(Employee::raw())
            .name("cursor")
            .scroll()
            .create_tx_model_cursor(None)
            .await
            .unwrap();

        assert!(cursor
            .stream(cursor.fetch_stmt().forward(10))
            .next()
            .await
            .unwrap()
            .is_err());

        let errs: Vec<Result<Employee, crate::model::Error>> = cursor
            .stream(cursor.fetch_stmt().forward(10))
            .collect()
            .await;
        assert_eq!(errs.len(), 1);
        cursor.close().await.unwrap().rollback().await.unwrap();
    }

    #[tokio::test]
    async fn test_selective_cursor() {
        let cursor = DeclareCursor::from(Employee::selective())
            .name("cursor")
            .create_tx_selective_cursor(None)
            .await;
        assert!(cursor.is_ok());
        let mut cursor = cursor.unwrap();

        let employed = cursor.fetch_one(cursor.fetch_stmt()).await;
        assert!(employed.is_ok());
        let employed = employed.unwrap();
        assert_eq!(employed.len(), 2);
        let cols = cursor.as_ref();
        let vals = cols
            .iter()
            .map(|col| employed.get(col).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            vals,
            vec![&Value::Int(1), &Value::String("bigboss".to_string())]
        );
        assert_eq!(
            cols.as_slice(),
            &[
                Column::new("employed", "id"),
                Column::new("employed", "name")
            ]
        );
        cursor.close().await.unwrap().rollback().await.unwrap();
    }
}
