use super::ToConnectionRequest;
use super::{FromRow, Query, Row};
use crate::{
    model::{Escape, Placeholders},
    prelude::*,
};
use std::{marker::PhantomData, sync::atomic::AtomicI64, time::Instant, vec};

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
            RELATIVE(n) | ABSOLUTE(n) => 1.eq(row_count),

            FORWARD(0) | BACKWARD(0) => 1.eq(row_count),
            FORWARD(n) | BACKWARD(n) => n.abs().eq(row_count),
            FORWARD_ALL | BACKWARD_ALL => 0.lt(row_count),
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

impl<T: Model> DeclareCursor<T> {
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

#[derive(Debug)]
pub struct ModelCursor<T: Model> {
    name: String,
    created: Instant,
    used: Instant,
    position: AtomicI64,
    fetched: AtomicI64,
    _marker: PhantomData<T>,
}

struct Fetch<'a>
where
    Self: Send + Sync,
{
    fetch: bool,
    direction: FetchDirection,
    cursor: &'a mut dyn Cursor,
}

impl<'a> ToSql for Fetch<'a> {
    fn to_sql(&self) -> String {
        format!(
            r#"{}{}FOR" {}""#,
            self.fetch.then(|| "FETCH").unwrap_or("MOVE"),
            self.direction.to_sql(),
            self.cursor.name().escape()
        )
    }
}

pub trait Cursor: Sync + Send {
    fn name(&self) -> &str;
    fn created(&self) -> Instant;
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

impl<T: Model + Send + Sync> Cursor for ModelCursor<T> {
    fn name(&self) -> &str {
        self.name.as_str()
    }
    fn created(&self) -> Instant {
        self.created
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

#[async_trait]
trait FetchInternal<'a>: Cursor + Send + Sync {
    async fn fetch_cursor(
        fetch: Fetch<'a>,
        req: impl ToConnectionRequest<'_> + Send,
    ) -> Result<Vec<tokio_postgres::Row>, super::Error> {
        let conn = req.to_connection_request()?.connection().unwrap();

        let rows = conn.query_cached(fetch.to_sql().as_str(), &[]).await?;
        fetch.cursor.update(fetch.direction, rows.len() as i64);
        Ok(rows)
    }
}
