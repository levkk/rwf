use super::column::{Column, ToColumn};
use super::picked::Picked;
use super::placeholders::Placeholders;
use super::select::Select;
use super::value::Value;
use super::{Escape, FromRow, Query, ToSql};
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
/// use rwf::model::temporary::{TemporaryQuery, ToTemporaryQuery};
/// use rwf::model::{ToSql, Model, Query};
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
        self.where_clause.add_offset(offset);
        self.combines.add_offset(offset);
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
        self.select.where_clause.add_offset(offset);
        self.select.combines.add_offset(offset);
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
    pub(super) fn offset(&self) -> i32 {
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

    pub fn with_query<T: FromRow>(&mut self, query: Query<T>, alias: impl ToString) -> i32 {
        match query {
            Query::Select(select) => self.add(select, alias, false),
            Query::Picked(picked) => self.add(picked, alias, false),
            Query::Raw { .. } => self.add(query, alias, false),
            _ => 0,
        }
    }
    pub fn with_recursive<T: FromRow>(&mut self, query: Query<T>, alias: impl ToString) -> i32 {
        match query {
            Query::Select(select) => self.add(select, alias, true),
            Query::Picked(picked) => self.add(picked, alias, true),
            Query::Raw { .. } => self.add(query, alias, true),
            _ => 0,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn last(&mut self) -> Option<&mut TemporaryQuery> {
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
