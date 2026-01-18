use super::{FromRow, Placeholders, Query, ToSql};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Combine<T: FromRow> {
    UNION(Query<T>),
    UNIONALL(Query<T>),
    INTERSECT(Query<T>),
    INTERSECTALL(Query<T>),
    EXCEPT(Query<T>),
    EXCEPTALL(Query<T>),
}
impl<T: FromRow> Combine<T> {
    /// # Example
    /// ```
    /// use rwf::model::{ToSql, Model, Query, FromRow};
    /// use rwf::model::combine::Combine;
    /// #[derive(rwf::macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize, Clone)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String,
    ///     mail: String
    /// }
    /// let union = Combine::union(User::find(1));
    /// assert!(union.is_ok());
    /// assert_eq!(union.unwrap().to_sql(), r#"UNION SELECT * FROM "users" WHERE "users"."id" = $1 LIMIT 1"#);
    /// ```
    pub fn union(q: Query<T>) -> Result<Self, super::Error> {
        match q {
            Query::Select(_) => Ok(Combine::UNION(q)),
            Query::Picked(_) => Ok(Combine::UNION(q)),
            _ => Err(super::Error::QueryError("Combined Querys are only defined for SELECT thus for Query::Select and Query::Picked".to_string(), q.to_sql()))

        }
    }
    /// # Example
    /// ```
    /// use rwf::model::{ToSql, Model, Query, FromRow};
    /// use rwf::model::combine::Combine;
    /// #[derive(rwf::macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize, Clone)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String,
    ///     mail: String
    /// }
    /// let union = Combine::union_all(User::find(1));
    /// assert!(union.is_ok());
    /// assert_eq!(union.unwrap().to_sql(), r#"UNION ALL SELECT * FROM "users" WHERE "users"."id" = $1 LIMIT 1"#);
    /// ```
    pub fn union_all(q: Query<T>) -> Result<Self, super::Error> {
        match q {
            Query::Select(_) => Ok(Combine::UNIONALL(q)),
            Query::Picked(_) => Ok(Combine::UNIONALL(q)),
            _ => Err(super::Error::QueryError("Combined Querys are only defined for SELECT thus for Query::Select and Query::Picked".to_string(), q.to_sql()))

        }
    }
    /// # Example
    /// ```
    /// use rwf::model::{ToSql, Model, Query, FromRow};
    /// use rwf::model::combine::Combine;
    /// #[derive(rwf::macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize, Clone)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String,
    ///     mail: String
    /// }
    /// let intersect = Combine::intersect(User::find(1));
    /// assert!(intersect.is_ok());
    /// assert_eq!(intersect.unwrap().to_sql(), r#"INTERSECT SELECT * FROM "users" WHERE "users"."id" = $1 LIMIT 1"#);
    /// ```
    pub fn intersect(q: Query<T>) -> Result<Self, super::Error> {
        match q {
            Query::Select(_) => Ok(Combine::INTERSECT(q)),
            Query::Picked(_) => Ok(Combine::INTERSECT(q)),
            _ => Err(super::Error::QueryError("Combined Querys are only defined for SELECT thus for Query::Select and Query::Picked".to_string(), q.to_sql()))

        }
    }
    /// # Example
    /// ```
    /// use rwf::model::{ToSql, Model, Query, FromRow};
    /// use rwf::model::combine::Combine;
    /// #[derive(rwf::macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize, Clone)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String,
    ///     mail: String
    /// }
    /// let intersect = Combine::intersect_all(User::find(1));
    /// assert!(intersect.is_ok());
    /// assert_eq!(intersect.unwrap().to_sql(), r#"INTERSECT ALL SELECT * FROM "users" WHERE "users"."id" = $1 LIMIT 1"#);
    /// ```
    pub fn intersect_all(q: Query<T>) -> Result<Self, super::Error> {
        match q {
            Query::Select(_) => Ok(Combine::INTERSECTALL(q)),
            Query::Picked(_) => Ok(Combine::INTERSECTALL(q)),
            _ => Err(super::Error::QueryError("Combined Querys are only defined for SELECT thus for Query::Select and Query::Picked".to_string(), q.to_sql()))

        }
    }
    /// # Example
    /// ```
    /// use rwf::model::{ToSql, Model, Query, FromRow};
    /// use rwf::model::combine::Combine;
    /// #[derive(rwf::macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize, Clone)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String,
    ///     mail: String
    /// }
    /// let exclude = Combine::except(User::find(1));
    /// assert!(exclude.is_ok());
    /// assert_eq!(exclude.unwrap().to_sql(), r#"EXCEPT SELECT * FROM "users" WHERE "users"."id" = $1 LIMIT 1"#);
    /// ```
    pub fn except(q: Query<T>) -> Result<Self, super::Error> {
        match q {
            Query::Select(_) => Ok(Combine::EXCEPT(q)),
            Query::Picked(_) => Ok(Combine::EXCEPT(q)),
            _ => Err(super::Error::QueryError("Combined Querys are only defined for SELECT thus for Query::Select and Query::Picked".to_string(), q.to_sql()))

        }
    }
    /// # Example
    /// ```
    /// use rwf::model::{ToSql, Model, Query, FromRow};
    /// use rwf::model::combine::Combine;
    /// #[derive(rwf::macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize, Clone)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String,
    ///     mail: String
    /// }
    /// let exclude = Combine::except_all(User::find(1));
    /// assert!(exclude.is_ok());
    /// assert_eq!(exclude.unwrap().to_sql(), r#"EXCEPT ALL SELECT * FROM "users" WHERE "users"."id" = $1 LIMIT 1"#);
    /// ```
    pub fn except_all(q: Query<T>) -> Result<Self, super::Error> {
        match q {
            Query::Select(_) => Ok(Combine::EXCEPTALL(q)),
            Query::Picked(_) => Ok(Combine::EXCEPTALL(q)),
            _ => Err(super::Error::QueryError("Combined Querys are only defined for SELECT thus for Query::Select and Query::Picked".to_string(), q.to_sql()))
        }
    }

    pub(super) fn add_offset(&mut self, offset: i32) {
        match self.deref_mut() {
            Query::Select(ref mut q) => q.where_clause.add_offset(offset),
            Query::Picked(ref mut q) => q.select.where_clause.add_offset(offset),
            _ => panic!("Combine is only defined for SELECT Statements"),
        }
    }

    pub(super) fn placeholders(&self) -> Placeholders {
        match self.deref() {
            Query::Select(q) => q.placeholders(),
            Query::Picked(q) => q.select.placeholders(),
            _ => panic!("Combine is only defined for SELECT Statements"),
        }
    }

    pub(super) fn take_with(&mut self) -> super::temporary::With {
        match self.deref_mut() {
            Query::Select(q) => std::mem::take(&mut q.with),
            Query::Picked(q) => std::mem::take(&mut q.select.with),
            _ => panic!("Combine is only defined for SELECT Statements"),
        }
    }
}

impl<T: FromRow> std::fmt::Display for Combine<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Combine::UNION(_) => write!(f, "UNION"),
            Combine::UNIONALL(_) => write!(f, "UNION ALL"),
            Combine::INTERSECT(_) => write!(f, "INTERSECT"),
            Combine::INTERSECTALL(_) => write!(f, "INTERSECT ALL"),
            Combine::EXCEPT(_) => write!(f, "EXCEPT"),
            Combine::EXCEPTALL(_) => write!(f, "EXCEPT ALL"),
        }
    }
}

impl<T: FromRow> std::ops::Deref for Combine<T> {
    type Target = Query<T>;
    fn deref(&self) -> &Self::Target {
        match self {
            Combine::UNION(q) => q,
            Combine::UNIONALL(q) => q,
            Combine::INTERSECT(q) => q,
            Combine::INTERSECTALL(q) => q,
            Combine::EXCEPT(q) => q,
            Combine::EXCEPTALL(q) => q,
        }
    }
}

impl<T: FromRow> std::ops::DerefMut for Combine<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Combine::UNION(q) => q,
            Combine::UNIONALL(q) => q,
            Combine::INTERSECT(q) => q,
            Combine::INTERSECTALL(q) => q,
            Combine::EXCEPT(q) => q,
            Combine::EXCEPTALL(q) => q,
        }
    }
}

impl<T: FromRow> ToSql for Combine<T> {
    fn to_sql(&self) -> String {
        use std::ops::Deref;
        format!("{} ({})", self, self.deref().to_sql())
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Combines<T: FromRow> {
    inner: Vec<Combine<T>>,
}

impl<T: FromRow> Default for Combines<T> {
    fn default() -> Self {
        Combines { inner: Vec::new() }
    }
}

impl<T: FromRow> ToSql for Combines<T> {
    fn to_sql(&self) -> String {
        if self.is_empty() {
            String::new()
        } else {
            format!(
                " {}",
                self.inner
                    .iter()
                    .map(|c| c.to_sql())
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
            )
        }
    }
}

impl<T: FromRow> Combines<T> {
    pub(super) fn add_query(&mut self, q: Combine<T>) {
        self.inner.push(q);
    }
    pub(super) fn placeholders_id(&self) -> i32 {
        self.inner
            .iter()
            .map(|combine| match combine.deref() {
                Query::Select(select) => select.where_clause().placeholders() as i32,
                Query::Picked(picked) => picked.select.where_clause().placeholders() as i32,
                _ => panic!("Combine is only implemented for SELECT Statements"),
            })
            .sum()
    }
    pub(super) fn inc_placeholders(&mut self) {
        self.inner
            .iter_mut()
            .for_each(|combine| combine.add_offset(1))
    }

    pub(super) fn add_offset(&mut self, offset: i32) {
        self.inner
            .iter_mut()
            .for_each(|combine| combine.add_offset(offset))
    }

    pub(super) fn placeholders(&self) -> Vec<Placeholders> {
        self.inner
            .iter()
            .map(|combine| combine.placeholders())
            .collect()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
