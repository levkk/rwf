use super::column::{Column, ToColumn};
use super::picked::Picked;
use super::placeholders::Placeholders;
use super::select::Select;
use super::value::Value;
use super::{Delete, Escape, FromRow, Insert, Query, ToSql, Update};
use serde::{Deserialize, Serialize};
use std::ops::AddAssign;
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Hash)]
pub struct Record {
    column: Column,
    value: Value,
}
/// A Query to create a temporary and named Record Set like in a WITH Statement or when creating a temporary Table
/// # Example
///```
/// use rwf::model::temporary::{TemporaryQuery, ToTemporaryQuery, WithQuery};
/// use rwf::model::{ToSql, Model, Query};
/// use rwf::model::select::FilterQuery;;
/// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
/// struct User {
///    id: Option<i64>,
///    name: String,
///    email: String
/// }
/// let all_users: rwf::model::Select<User> = rwf::model::Select::new(User::table_name(), User::primary_key()).filter_gt("id", 5);
/// let temp = all_users.clone().to_temporary("allusr", 0);
/// assert_eq!(temp.to_sql(), r#""allusr" AS (SELECT * FROM "users" WHERE "users"."id" > $1)"#);
/// assert_eq!(all_users.placeholders().get(1), temp.placeholders().get(1));
/// assert_eq!(all_users.placeholders().values().len(), temp.placeholders().values().len())
///```
///
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TemporaryQuery {
    alias: String,
    recursive: bool,
    as_stmt: String,
    fields: Vec<Column>,
    placeholders: Placeholders,
    offset: i32,
}

impl TemporaryQuery {
    pub fn placeholders(&self) -> Placeholders {
        self.placeholders.clone()
    }
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }
    pub fn offset(&self) -> i32 {
        self.offset
    }
    pub fn fields(&mut self, fields: Vec<Column>) -> () {
        self.fields = fields
            .into_iter()
            .map(|col| Column::name(col.get_name()))
            .collect();
    }
    pub fn fields_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

pub trait ToTemporaryQuery {
    fn to_temporary(self, alias: impl ToString, offset: i32) -> TemporaryQuery;
}

impl<T: FromRow> ToTemporaryQuery for Select<T> {
    fn to_temporary(mut self, alias: impl ToString, offset: i32) -> TemporaryQuery {
        self.add_offset(offset);
        let offset =
            offset + self.where_clause.placeholders() as i32 + self.combines.placeholders_id();
        let alias = alias.to_string();
        TemporaryQuery {
            alias: alias.clone(),
            recursive: false,
            as_stmt: self.to_sql(),
            placeholders: self.placeholders(),
            fields: self
                .columns
                .columns
                .iter()
                .map(|col| col.get_name().to_column().qualify(alias.clone()))
                .collect(),
            offset,
        }
    }
}
impl<T: FromRow> ToTemporaryQuery for Picked<T> {
    fn to_temporary(mut self, alias: impl ToString, offset: i32) -> TemporaryQuery {
        self.add_offset(offset);
        let alias = alias.to_string();
        let offset = offset
            + self.select.where_clause.placeholders() as i32
            + self.select.combines.placeholders_id();
        TemporaryQuery {
            alias: alias.clone(),
            recursive: false,
            as_stmt: self.to_sql(),
            placeholders: self.select.placeholders(),
            fields: self
                .columns()
                .into_iter()
                .map(|col| col.get_name().to_column().qualify(alias.clone()))
                .collect(),
            offset,
        }
    }
}

impl<T: FromRow> ToTemporaryQuery for Update<T> {
    fn to_temporary(mut self, alias: impl ToString, offset: i32) -> TemporaryQuery {
        self.add_offset(offset);
        let offset = offset + self.get_statement_offset();
        TemporaryQuery {
            alias: alias.to_string(),
            recursive: false,
            as_stmt: self.to_sql(),
            fields: vec![],
            placeholders: self.placeholders,
            offset,
        }
    }
}

impl<T: FromRow> ToTemporaryQuery for Delete<T> {
    fn to_temporary(mut self, alias: impl ToString, offset: i32) -> TemporaryQuery {
        self.add_offset(offset);
        let offset = offset + self.get_statement_offset();
        TemporaryQuery {
            alias: alias.to_string(),
            recursive: false,
            as_stmt: self.to_sql(),
            fields: vec![],
            placeholders: self.placeholders,
            offset,
        }
    }
}

impl<T: FromRow> ToTemporaryQuery for Insert<T> {
    fn to_temporary(mut self, alias: impl ToString, offset: i32) -> TemporaryQuery {
        self.add_offset(offset);
        let offset = offset + self.get_statement_offset();
        TemporaryQuery {
            alias: alias.to_string(),
            recursive: false,
            as_stmt: self.to_sql(),
            fields: vec![],
            placeholders: self.placeholders(),
            offset: 0,
        }
        .to_temporary("", offset)
    }
}

impl<T: FromRow> ToTemporaryQuery for Query<T> {
    fn to_temporary(self, alias: impl ToString, mut offset: i32) -> TemporaryQuery {
        match self {
            Query::Select(select) => select.to_temporary(alias, offset),
            Query::Picked(picked) => picked.to_temporary(alias, offset),
            Query::Raw {
                mut query,
                placeholders,
            } => {
                let mut val_placeholders = (1..placeholders.id()).into_iter().collect::<Vec<i32>>();
                val_placeholders.reverse();
                for (idx, placeholder) in val_placeholders.iter().enumerate() {
                    query = query.replace(
                        format!("${}", idx + 1).as_str(),
                        format!("${}", placeholder + offset).as_str(),
                    );
                    offset += 1;
                }
                TemporaryQuery {
                    alias: alias.to_string(),
                    recursive: false,
                    as_stmt: query,
                    fields: vec![],
                    placeholders,
                    offset,
                }
            }
            Query::Update(update) => update.to_temporary(alias, offset),
            Query::Delete(delete) => delete.to_temporary(alias, offset),
            Query::Insert(insert) => insert.to_temporary(alias, offset),
            _ => unimplemented!("ToTemporaryQuery is only implemented for select or picked or raw"),
        }
    }
}

impl ToTemporaryQuery for TemporaryQuery {
    fn to_temporary(self, _alias: impl ToString, mut offset: i32) -> TemporaryQuery {
        let mut stmt = String::new();
        let mut dolla_seen = false;
        for char in self.as_stmt.chars() {
            if char.eq(&'$') {
                stmt.push('$');
                dolla_seen = true;
            } else if char.is_numeric() && dolla_seen {
                continue;
            } else if !char.is_numeric() && dolla_seen {
                offset += 1;
                stmt.add_assign(offset.to_string().as_str());
                dolla_seen = false;
                stmt.push(char);
            } else {
                stmt.push(char);
            }
        }
        if dolla_seen {
            offset += 1;
            stmt.add_assign(offset.to_string().as_str());
        }
        TemporaryQuery {
            alias: self.alias,
            recursive: self.recursive,
            as_stmt: stmt,
            fields: self.fields,
            placeholders: self.placeholders,
            offset,
        }
    }
}
impl ToSql for TemporaryQuery {
    fn to_sql(&self) -> String {
        if self.recursive {
            format!(
                r#"RECURSIVE "{}"({}) AS ({})"#,
                self.alias.escape(),
                self.fields
                    .iter()
                    .map(|col| col.get_name())
                    .collect::<Vec<_>>()
                    .join(", "),
                self.as_stmt.trim()
            )
        } else {
            format!(r#""{}" AS ({})"#, self.alias.escape(), self.as_stmt.trim())
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct With(Vec<TemporaryQuery>);

impl With {
    fn offset(&self) -> i32 {
        if let Some(with) = self.0.last() {
            with.offset
        } else {
            0
        }
    }
    pub(super) fn placeholders(&self) -> Vec<Placeholders> {
        self.0.iter().map(|c| c.placeholders()).collect()
    }

    fn add(&mut self, query: impl ToTemporaryQuery, alias: impl ToString, recurive: bool) -> i32 {
        let offset = self.offset();
        self.0
            .push(query.to_temporary(alias, offset).recursive(recurive));
        self.offset() - offset
    }

    pub(super) fn extend(&mut self, other: Self) -> i32 {
        let mut offset = 0;
        for query in other.0.into_iter() {
            let _offset = self.offset();
            self.0.push(query.to_temporary("", _offset));
            offset += self.offset() - _offset;
        }
        offset
    }
    fn merge_with<T: FromRow>(&mut self, query: &mut Query<T>) -> i32 {
        match query {
            Query::Select(select) => self.extend(std::mem::take(select.with_statements_mut())),
            Query::Picked(picked) => self.extend(std::mem::take(picked.with_statements_mut())),
            Query::Update(update) => self.extend(std::mem::take(update.with_statements_mut())),
            Query::Delete(delete) => self.extend(std::mem::take(delete.with_statements_mut())),
            Query::Insert(insert) => self.extend(std::mem::take(insert.with_statements_mut())),
            Query::InsertIfNotExists { select, .. } => {
                self.extend(std::mem::take(select.with_statements_mut()))
            }
            _ => 0,
        }
    }

    /// Convert a `Query<T>` to a temporary statement via the ``ToTemporaryQuery' trait
    /// Before that the `Query<T>` is checked for any WITH statements and if some are found, they are appended to own ones.
    /// At least the difference beetween the old Offset and the new is returned.
    ///
    /// This method is called by the `TemporaryQuery` trait in the `with` method, no need to call it directly.
    fn with_query<T: FromRow>(&mut self, mut query: Query<T>, alias: impl ToString) -> i32 {
        let offset = self.merge_with(&mut query);
        match query {
            Query::Select(select) => self.add(select, alias, false) + offset,
            Query::Picked(picked) => self.add(picked, alias, false) + offset,
            Query::Update(update) => self.add(update, alias, false) + offset,
            Query::Delete(delete) => self.add(delete, alias, false) + offset,
            Query::Insert(insert) => self.add(insert, alias, false) + offset,
            Query::InsertIfNotExists { select, .. } => self.add(select, alias, false) + offset,
            Query::Raw { .. } => self.add(query, alias, false),
        }
    }

    /// Convert a `Query<T>` to a recursive temporary statement via the ``ToTemporaryQuery' trait
    /// Before that the `Query<T>` is checked for any WITH statements and if some are found, they are appended to own ones.
    /// Last but not least the difference beetween the old Offset and the new is returned.
    ///
    /// This method is called by the `TemporaryQuery` trait in the `with_recursive` method, no need to call it directly.
    /// Recursive Queries must noot contain Data Modifying Statements!
    fn with_recursive<T: FromRow>(&mut self, mut query: Query<T>, alias: impl ToString) -> i32 {
        let offset = self.merge_with(&mut query);
        match query {
            Query::Select(select) => self.add(select, alias, true) + offset,
            Query::Picked(picked) => self.add(picked, alias, true) + offset,
            Query::InsertIfNotExists { select, .. } => self.add(select, alias, false) + offset,
            Query::Raw { .. } => self.add(query, alias, true),
            _ => 0,
        }
    }
    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    fn last(&mut self) -> Option<&mut TemporaryQuery> {
        self.0.last_mut()
    }
}
impl ToSql for With {
    fn to_sql(&self) -> String {
        if self.is_empty() {
            String::new()
        } else {
            let querys = self
                .0
                .iter()
                .map(|c| c.to_sql())
                .collect::<Vec<_>>()
                .join(", ");
            format!("WITH {} ", querys)
        }
    }
}

pub trait WithQuery {
    /// Get a reference to the with statements the current Query holds. Only for internal purposes
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// #[derive(Clone, rwf::macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize)]
    /// struct Product {
    ///     id: Option<i64>,
    ///     name: String,
    ///     stock: i16,
    ///     price: f32
    /// }
    /// let query = Product::all().filter_lt("stock", 100).add_except(Product::find(1));
    ///
    /// assert!(!query.with_statements().to_sql().starts_with("WITH"));
    /// let query = query.with(Product::all().filter_gt("price", 999.99), "expansive_:products");
    /// assert!(query.with_statements().to_sql().starts_with("WITH"));
    /// ```
    fn with_statements(&self) -> &With;
    /// Get the offset the query must add to all Placeholders caused by the with Queries
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// #[derive(Clone, rwf::macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize)]
    /// struct Product {
    ///     id: Option<i64>,
    ///     name: String,
    ///     stock: i16,
    ///     price: f32
    /// }
    /// let query = Product::all().filter_lt("stock", 100).add_except(Product::find(1));
    /// let offset = query.get_with_offset();
    /// assert_eq!(offset, 0);
    /// let query = query.with(Product::all().filter_gt("price", 999.99), "expansive_:products");
    /// assert_eq!(query.get_with_offset(), 1);
    /// ```
    fn get_with_offset(&self) -> i32 {
        self.with_statements().offset()
    }
    /// Get a mutable reference to the with statements the current Query holds
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// #[derive(Clone, rwf::macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize)]
    /// struct Product {
    ///     id: Option<i64>,
    ///     name: String,
    ///     stock: i16,
    ///     price: f32
    /// }
    /// let mut query = Product::all().filter_lt("stock", 100).add_except(Product::find(1)).with(Product::all().filter_gt("price", 999.99), "expansive_:products");
    /// assert!(!query.with_is_empty());
    /// let _ = std::mem::take(query.with_statements_mut());
    /// assert!(query.with_is_empty());
    /// ```
    fn with_statements_mut(&mut self) -> &mut With;
    /// Get the offset the query causes by itself (or by combined queries)
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// #[derive(Clone, rwf::macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize)]
    /// struct Product {
    ///     id: Option<i64>,
    ///     name: String,
    ///     stock: i16,
    ///     price: f32
    /// }
    /// let query = Product::all().filter_lt("stock", 100);
    /// assert_eq!(query.get_statement_offset(), 1);
    /// let query = query.add_except(Product::find(1));
    /// assert_eq!(query.get_statement_offset(), 2);
    /// let query = query.with(Product::all().filter_gt("price", 999.99), "expansive_:products");
    /// assert_eq!(query.get_statement_offset(), 2);
    /// ```
    fn get_statement_offset(&self) -> i32;
    /// Increate the placeholders of the WHERE CLAUSE.
    /// # Example
    /// ```
    /// use rwf::model::Placeholders;
    /// use rwf::model::prelude::*;
    /// #[derive(Clone, rwf::macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize)]
    /// struct Product {
    ///     id: Option<i64>,
    ///     name: String,
    ///     stock: i16,
    ///     price: f32
    /// }
    /// let mut query = Product::all().filter_lt("stock", 100);
    /// assert_eq!(query.get_where_clause().placeholders(), 1);
    /// query.add_offset(1);
    /// assert_eq!(query.get_where_clause().placeholders(), 1);
    /// ```
    fn add_offset(&mut self, offset: i32);
    /// Construct a WITH Query and make it available to the current one.
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// use rwf::model::join::Join;
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// #[has_many(Order)]
    /// struct User {
    ///     id: Option<i64>,
    ///     name: String,
    ///     low_credit: bool
    /// }
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// #[belongs_to(User)]
    /// struct Order {
    ///     id: Option<i64>,
    ///     user_id: i64,
    ///     expensive: bool
    /// }
    /// assert_eq!(
    ///     User::all().with(User::all().join::<Order>().filter("low_credit", true.to_value()).filter(Order::column("expensive"), true.to_value()), "to_investigate").add_join(Join::new(User::table_name(), "to_investigate", "id", "id")).to_sql(),
    ///     r#"WITH "to_investigate" AS (SELECT "users".* FROM "users" INNER JOIN "orders" ON "users"."id" = "orders"."user_id" WHERE "users"."low_credit" = $1 AND "orders"."expensive" = $2) SELECT "users".* FROM "users" INNER JOIN "to_investigate" ON "to_investigate"."id" = "users"."id""#
    /// )
    /// ```
    fn with<U: FromRow>(mut self, other: Query<U>, alias: impl ToString) -> Self
    where
        Self: Sized,
    {
        let offset = self.with_statements_mut().with_query(other, alias);
        self.add_offset(offset);
        self
    }
    /// Construct a recursive WITH Statement and make it available to the current one
    /// # Example
    /// ```
    /// use rwf::model::prelude::*;
    /// use rwf::model::join::Join;
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// #[belongs_to(Order)]
    /// #[has_many(OrderItem)]
    /// struct Order {
    ///     id: Option<i64>,
    ///     order_id: Option<i64>
    /// }
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// struct OrderItem {
    ///     id: Option<i64>,
    ///     order_id: i64,
    ///     item: String,
    ///     price: f32
    /// }
    /// let mut query = OrderItem::all().with_recursive(Order::find_by("order_id", Value::Null).add_union(Order::all().add_join(Join::new(Order::table_name(), "recurse", "id", "order_id"))), "recurse").add_join(Join::new(OrderItem::table_name(), "recurse", "id", Order::foreign_key()));
    /// if let Some(last) = query.last_with() {
    ///     if last.fields_empty() {last.fields(Order::all_columns())}
    /// }
    /// assert_eq!(
    ///     query.to_sql(),
    ///     r#"WITH RECURSIVE "recurse"(id, order_id) AS ((SELECT * FROM "orders" WHERE "orders"."order_id" IS NULL LIMIT 1) UNION (SELECT "orders".* FROM "orders" INNER JOIN "recurse" ON "recurse"."id" = "orders"."order_id")) SELECT "order_items".* FROM "order_items" INNER JOIN "recurse" ON "recurse"."id" = "order_items"."order_id""#
    /// )
    ///
    /// ```
    fn with_recursive<U: FromRow>(mut self, other: Query<U>, alias: impl ToString) -> Self
    where
        Self: Sized,
    {
        let offset = self.with_statements_mut().with_recursive(other, alias);
        self.add_offset(offset);
        self
    }
    /// Get all `Placeholders` of the current query no matter if the `Placeholders` is part of `Self` or `WIth` or `Combines`
    /// # Example
    /// ```
    /// use rwf::model::Placeholders;
    /// use rwf::model::prelude::*;
    /// #[derive(Clone, rwf::macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize)]
    /// struct Product {
    ///     id: Option<i64>,
    ///     name: String,
    ///     stock: i16,
    ///     price: f32
    /// }
    /// let query = Product::all().filter_lt("stock", 100).with(Product::all().filter_gt("price", 999.99), "expansive_:products").add_except(Product::find(1));
    /// assert_eq!(
    ///     query.placeholders(),
    ///     Placeholders::from(vec![999.99.to_value(), 100.to_value(), 1.to_value()])
    /// )
    /// ```
    fn placeholders(&self) -> Placeholders;
    /// Checks if any with statements exists in the current query
    /// # Example
    /// ```
    /// use rwf::model::Placeholders;
    /// use rwf::model::prelude::*;
    /// #[derive(Clone, rwf::macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize)]
    /// struct Product {
    ///     id: Option<i64>,
    ///     name: String,
    ///     stock: i16,
    ///     price: f32
    /// }
    /// let query = Product::all().filter_lt("stock", 100);
    /// assert!(query.with_is_empty());
    /// let query = query.with(Product::all().filter_gt("price", 999.99), "expansive_:products");
    /// assert!(!query.with_is_empty());
    /// ```
    fn with_is_empty(&self) -> bool {
        self.with_statements().is_empty()
    }
    /// Get the last statement created for the current Query.
    /// Main purpose is, to check if columns are set correctly in a recursive statement (or update them otherwise)
    /// # Example
    /// ```
    /// use rwf::model::Placeholders;
    /// use rwf::model::prelude::*;
    /// #[derive(Clone, rwf::macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize)]
    /// struct Product {
    ///     id: Option<i64>,
    ///     name: String,
    ///     stock: i16,
    ///     price: f32
    /// }
    /// let mut query = Product::all().filter_lt("stock", 100);
    /// assert!(query.last_with().is_none());
    /// query = query.with(Product::all().filter_gt("price", 999.99), "expansive_products");
    /// assert!(query.last_with().is_some());
    /// ```
    fn last_with(&mut self) -> Option<&mut TemporaryQuery> {
        self.with_statements_mut().last()
    }
}
