use super::pool::Transaction;
use super::ToConnectionRequest;
use super::{FromRow, Query, Row};
use crate::config::get_config;
use crate::model::{ConnectionGuard, Error};
use crate::{
    model::{Escape, Placeholders},
    prelude::*,
};
use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::thread::yield_now;
use std::{marker::PhantomData, sync::atomic::AtomicI64, time::Instant, vec};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum FetchDirection {
    NEXT,
    PRIOR,
    FIRST,
    LAST,
    ABSOLUTE(i64),
    RELATIVE(i64),
    FORWARD(i64),
    FORWARD_ALL,
    BACKWARD(i64),
    BACKWARD_ALL,
}

impl Default for FetchDirection {
    fn default() -> Self {
        Self::NEXT
    }
}

impl std::fmt::Display for FetchDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NEXT => write!(f, "NEXT"),
            Self::PRIOR => write!(f, "PRIOR"),
            Self::FIRST => write!(f, "FIRST"),
            Self::LAST => write!(f, "LAST"),
            Self::ABSOLUTE(n) => write!(f, "{} {}", "ABSOLUTE", n),
            Self::RELATIVE(n) => write!(f, "{} {}", "RELATIVE", n),
            Self::FORWARD(n) => write!(f, "{} {}", "FORWARD", n),
            Self::FORWARD_ALL => write!(f, "FORWARD ALL"),
            Self::BACKWARD(n) => write!(f, "{} {}", "BACKWARD", n),
            Self::BACKWARD_ALL => write!(f, "BACKWARD ALL"),
        }
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
            FORWARD_ALL | BACKWARD_ALL => 0.lt(row_count),
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
                FORWARD_ALL => FORWARD(*row_count),
                BACKWARD_ALL => BACKWARD(*row_count),
                fd => *fd,
            }
        } else {
            match self {
                NEXT | FIRST | LAST | PRIOR | FORWARD_ALL | BACKWARD_ALL => RELATIVE(0),
                ABSOLUTE(_) => FORWARD_ALL,
                RELATIVE(n) => {
                    if 0.eq(n) {
                        RELATIVE(0)
                    } else if 0.lt(n) {
                        FORWARD_ALL
                    } else {
                        BACKWARD_ALL
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
pub enum Sensitivity {
    INSENSITIVE,
    ASENSITIVE,
    SENSITIVE,
}
impl Default for Sensitivity {
    fn default() -> Self {
        Self::INSENSITIVE
    }
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

impl ToSql for FetchDirection {
    fn to_sql(&self) -> String {
        format!(" {} ", self)
    }
}

impl ToSql for Sensitivity {
    fn to_sql(&self) -> String {
        match self {
            Self::SENSITIVE => unimplemented!("Postgres has no support for SENSITIVE Cursors. SENSITIVE is only implemented for compability with the SQL Standard"),
            sensitivity => format!(" {} ", sensitivity.to_string())
        }
    }
}

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
    pub fn insensitive(mut self) -> Self {
        self.sensitivity = Sensitivity::INSENSITIVE;
        self
    }
    pub fn asensitive(mut self) -> Self {
        self.sensitivity = Sensitivity::ASENSITIVE;
        self
    }
    pub fn hold(mut self) -> Self {
        self.hold = !self.hold;
        self
    }
    pub fn scroll(mut self) -> Self {
        self.scroll = !self.scroll;
        self
    }
    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }
    pub fn placeholders(&self) -> &Placeholders {
        self.query.get_placeholders()
    }
    pub async fn create_model_cursor(
        self,
        conn: impl ToConnectionRequest<'_>,
    ) -> Result<ModelCursor<T>, Error> {
        let conn = conn.to_connection_request()?.connection().unwrap();
        conn.query_cached(self.to_sql().as_str(), &[]).await?;
        let cur = ModelCursor::from(self);
        Ok(cur)
    }
    pub async fn create_tx_model_cursor(self) -> Result<TxModelCursor<T>, Error> {
        let mut tx = Pool::pool().transaction().await?;
        self.create_model_cursor(&mut tx)
            .await
            .map(|mc| TxModelCursor { inner: mc, tx })
    }
}

impl<T: Model> ToSql for DeclareCursor<T> {
    fn to_sql(&self) -> String {
        format!(
            r#"DECLARE "{}" BINARY{}{} SCROLL CURSOR {} HOLD FOR {}"#,
            self.name.escape(),
            self.sensitivity.to_sql(),
            self.scroll.then(|| "").unwrap_or("NO"),
            self.hold.then(|| "WITH").unwrap_or("WITHOUT"),
            self.query.to_sql()
        )
    }
}

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
#[derive(Debug)]
pub struct ModelCursor<T>
where
    T: Model + Send + Sync,
    Self: Send + Sync,
{
    meta: CursorMeta,
    used: Instant,
    position: AtomicI64,
    fetched: AtomicI64,
    _marker: PhantomData<T>,
}
impl<T> From<DeclareCursor<T>> for ModelCursor<T>
where
    T: Model + Send + Sync,
{
    fn from(value: DeclareCursor<T>) -> Self {
        Self {
            meta: CursorMeta::from(value),
            used: Instant::now(),
            position: AtomicI64::new(0),
            fetched: AtomicI64::new(0),
            _marker: PhantomData,
        }
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
/*
impl<T> Deref for TxModelCursor<T>
where
    T: Model + Send + Sync,
{
    type Target = ModelCursor<T>;
    fn deref(&self) -> &Self::Target {
       &self.inner
    }
}
impl<T> DerefMut for TxModelCursor<T>
where
    T: Model + Send + Sync,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
*/

impl<T> std::ops::Deref for TxModelCursor<T>
where
    T: Model + Send + Sync + 'static,
{
    type Target = dyn Cursor;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T> std::ops::DerefMut for TxModelCursor<T>
where
    T: Model + Send + Sync + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> TransactionCursor for TxModelCursor<T>
where
    T: Model + Send + Sync + 'static,
{
    fn tx(&mut self) -> &mut Transaction {
        &mut self.tx
    }
}
impl<T> TxModelCursor<T>
where
    T: Model + Send + Sync + 'static,
{
    pub fn make_fetchable(self) -> Arc<Mutex<Self>> {
        self.fetched();
        Arc::new(Mutex::new(self))
    }
    pub fn make_dyn_fetchable(self) -> Arc<Mutex<dyn TransactionCursor>> {
        self.make_fetchable()
    }
}

impl ToFetch for Arc<Mutex<dyn TransactionCursor>> {
    fn to_fetch(&self, direction: FetchDirection) -> Fetch {
        Fetch {
            direction,
            cursor: self.clone(),
            fetch: true,
        }
    }
}

struct Fetch
where
    Self: Send + Sync,
{
    fetch: bool,
    direction: FetchDirection,
    cursor: Arc<Mutex<dyn TransactionCursor>>,
}

impl Fetch {
    fn toggle_fetch(&mut self) -> () {
        self.fetch = !self.fetch;
    }
    async fn fetch_cursor(&mut self) -> Result<Vec<tokio_postgres::Row>, Error> {
        let query = self.to_sql();
        if get_config().general.log_queries {
            info!("Fetch Cursor: {}", query)
        }
        let cursor = &mut self.cursor.lock().await;
        let rows = match cursor.tx().query_cached(query.as_str(), &[]).await {
            Ok(rows) => rows,
            Err(Error::RecordNotFound) => Vec::new(),
            Err(e) => {
                error!(
                    "Error while executing FETCH on {} -- Error -> {:#?}",
                    cursor.name(),
                    e
                );
                return Err(e);
            }
        };
        cursor.update(self.direction, rows.len() as i64);
        Ok(rows)
    }
    async fn move_cursor(&mut self) -> Result<bool, Error> {
        let query = self.to_sql();

        if get_config().general.log_queries {
            info!("Move Cursor: {}", query);
        }
        let cursor = &mut self.cursor.lock().await;
        let pos_ok = match cursor.tx().query_cached(query.as_str(), &[]).await {
            Ok(_) => {
                let direction = std::mem::replace(&mut self.direction, FetchDirection::RELATIVE(0));
                let query = self.to_sql();
                if get_config().general.log_queries {
                    info!(
                        "Check if Curso {} has a valid Position after moving",
                        cursor.name()
                    );
                }
                if let Ok(rows) = cursor.tx().query_cached(query.as_str(), &[]).await {
                    if rows.len() == 1 {
                        let _ = std::mem::replace(&mut self.direction, direction);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Err(e) => {
                error!(
                    "Error while moving Cursor {} -- Error -> {:?}",
                    cursor.name(),
                    e
                );
                return Err(e);
            }
        };
        if pos_ok {
            cursor.update_used();
            cursor.update_position(self.direction);
        } else {
            warn!("Failed to update Cutsor Position for {}", cursor.name());
        }
        Ok(pos_ok)
    }
    async fn execute(&mut self) -> Result<Vec<tokio_postgres::Row>, Error> {
        if self.fetch {
            self.fetch_cursor().await
        } else {
            self.move_cursor().await.map(|_| Vec::new())
        }
    }
}

trait ToFetch {
    fn to_fetch(&self, direction: FetchDirection) -> Fetch;
}

impl ToSql for Fetch {
    fn to_sql(&self) -> String {
        format!(
            r#"{}{}FOR" {}""#,
            self.fetch.then(|| "FETCH").unwrap_or("MOVE"),
            self.direction.to_sql(),
            self.cursor.blocking_lock().name().escape()
        )
    }
}

pub trait Cursor: Sync + Send {
    fn meta(&self) -> &CursorMeta;
    fn scrollable(&self) -> bool {
        self.meta().is_scroll_able
    }
    fn with_hold(&self) -> bool {
        self.meta().hold
    }
    fn insensitive(&self) -> bool {
        Sensitivity::INSENSITIVE.eq(&self.meta().sensitivity)
    }
    fn asensitive(&self) -> bool {
        Sensitivity::ASENSITIVE.eq(&self.meta().sensitivity)
    }
    fn name(&self) -> &str {
        self.meta().name.as_str()
    }
    fn created(&self) -> Instant {
        self.meta().created
    }
    fn last_used(&self) -> Instant;
    fn position(&self) -> i64;
    fn fetched(&self) -> i64;
    fn update(&mut self, fd: FetchDirection, row_count: i64) -> () {
        self.update_used();
        self.update_fetched(row_count);
        self.update_position(fd.to_position_update(&row_count));
    }
    fn update_position(&mut self, fd: FetchDirection) -> () {
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
            FORWARD_ALL => cur.swap(i64::MAX, order),
            BACKWARD_ALL => cur.swap(0, order),
        };
    }
    fn update_fetched(&mut self, count: i64) -> () {
        self.get_fetched_mut()
            .fetch_add(count, std::sync::atomic::Ordering::Relaxed);
    }
    fn update_used(&mut self) -> ();
    fn get_position_mut(&mut self) -> &mut AtomicI64;
    fn get_fetched_mut(&mut self) -> &mut AtomicI64;
}
pub trait TransactionCursor
where
    Self: Send + Sync + Deref<Target = dyn Cursor> + DerefMut<Target = dyn Cursor> + 'static,
{
    fn tx(&mut self) -> &mut Transaction;
}
#[async_trait]
pub trait ConnectionCursor<'a>: Send + Sync {
    type Conn: ToConnectionRequest<'a>;
    type Output;

    fn conn(&'a mut self) -> Self::Conn;
    fn cursor(&'a mut self) -> &'a mut dyn Cursor;
}

impl<T: Model + Send + Sync> Cursor for ModelCursor<T> {
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
    fn update_used(&mut self) -> () {
        self.used = Instant::now();
    }
    fn get_position_mut(&mut self) -> &mut AtomicI64 {
        &mut self.position
    }
    fn get_fetched_mut(&mut self) -> &mut AtomicI64 {
        &mut self.fetched
    }
}
/*
struct TxModelCursor<T: Model, C: Cursor> where Self: Send + Sync {
    cursor: C,
    tx: Transaction,
    _marker: PhantomData<T>
}

impl<T: Model + Send + Sync, C: Cursor> Cursor for TxModelCursor<T, C> {
    fn meta(&self) -> &CursorMeta { &self.cursor.meta() }
    fn name(&self) -> &str {
        self.cursor.name()
    }
    fn created(&self) -> Instant {
        self.cursor.created()
    }
    fn last_used(&self) -> Instant {
        self.cursor.last_used()
    }
    fn position(&self) -> i64 {
        self.cursor.position()
    }
    fn fetched(&self) -> i64 {
        self.cursor.fetched()
    }
    fn update_used(&mut self) -> () {
        self.cursor.update_used()
    }
    fn get_position_mut(&mut self) -> &mut AtomicI64 {
        self.cursor.get_position_mut()
    }
    fn get_fetched_mut(&mut self) -> &mut AtomicI64 {
       self.cursor.get_fetched_mut()
    }
}


impl<'a, T: Model + Send + Sync, C: Cursor + Send + Sync> ConnectionCursor<'a> for TxModelCursor<T, C>
{
    type Conn = &'a mut Transaction;
    type Output = T;

    fn cursor(&'a mut self) ->  &'a mut dyn Cursor {
        &mut self.cursor
    }
    fn conn(&'a mut self) -> &'a mut Transaction {
       &mut self.tx
    }
}

impl<'a, T: Model + Send + Sync, C: Cursor>  ToFetch<'a> for TxModelCursor<T, C> {
    fn to_fetch(&'a mut self, direction: FetchDirection) -> Fetch<'a> {

        let cur = self.cursor;
        let tx = self.tx.;
        let cc  self as &'a mut dyn ConnectionCursor;
        Fetch { conn, cursor, direction, fetch: true }
    }
}*/
