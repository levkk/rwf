//! Implements joining tables in a `SELECT` query.

use super::{Column, Escape, Model, ToColumn, ToSql};
use std::fmt::Formatter;
use std::marker::PhantomData;

/// Type of relationship between models.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Copy)]
pub enum AssociationType {
    /// Many-to-one relationship.
    BelongsTo,
    /// One-to-many relationship.
    HasMany,
    /// One-to-one relationship.
    HasOne,
}

/// Declare a relationship between model `T` and `Self`.
///
/// # Example
///
/// Declare a many-to-one relationship between `Order` and `User`:
///
/// ```
/// #[derive(Clone, Default)]
/// struct User {}
///
/// #[derive(Clone, Default)]
/// struct Order {}
///
/// ```
pub trait Association<T: Model>: Model {
    fn association_type() -> AssociationType {
        AssociationType::BelongsTo
    }

    fn belongs_to() -> bool {
        Self::association_type() == AssociationType::BelongsTo
    }

    fn has_many() -> bool {
        Self::association_type() == AssociationType::HasMany
    }

    fn construct_left_join() -> Join {
        Self::construct_join().replace_kind(JoinKind::Left)
    }

    fn construct_outer_join() -> Join {
        Self::construct_join().replace_kind(JoinKind::Outer)
    }
    fn construct_join() -> Join {
        use AssociationType::*;

        match Self::association_type() {
            // INNER JOIN "users" ON "users"."id" = "orders"."user_id"
            BelongsTo => {
                let table_name = Self::table_name().to_string();
                let table_column = Column::new(T::table_name(), T::primary_key());
                let foreign_column = Column::new(Self::table_name(), T::foreign_key());
                Join {
                    kind: JoinKind::Inner,
                    table_name,
                    table_column,
                    foreign_column,
                    alias: None,
                }
            }

            // INNER JOIN "orders" ON "orders"."user_id" = "users"."id"
            // HasOne is enforced by having a UNIQUE index on the foreign key.
            HasMany | HasOne => {
                let table_name = Self::table_name().to_string();
                let table_column = Column::new(T::table_name(), Self::foreign_key());
                let foreign_column = Column::new(Self::table_name(), Self::primary_key());
                Join {
                    kind: JoinKind::Inner,
                    table_name,
                    table_column,
                    foreign_column,
                    alias: None,
                }
            }
        }
    }
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Copy,
    Clone,
    crate::prelude::Deserialize,
    crate::prelude::Serialize,
)]
pub enum JoinKind {
    Inner,
    Left,
    Outer,
}

impl std::fmt::Display for JoinKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            JoinKind::Inner => write!(f, "INNER JOIN"),
            JoinKind::Left => write!(f, "LEFT JOIN"),
            JoinKind::Outer => write!(f, "OUTER JOIN"),
        }
    }
}

#[derive(Debug, Clone, crate::prelude::Deserialize, crate::prelude::Serialize, PartialEq, Eq)]
pub struct Join {
    pub(super) kind: JoinKind,
    pub(super) table_name: String,
    pub(super) table_column: Column,
    pub(super) foreign_column: Column,
    pub(super) alias: Option<String>,
}

impl Join {
    pub fn new(
        name: impl ToString,
        table_name: impl ToString,
        table_column: impl ToColumn,
        foreign_column: impl ToColumn,
    ) -> Self {
        Self {
            kind: JoinKind::Inner,
            table_name: table_name.to_string(),
            table_column: table_column.to_column().qualify(table_name.to_string()),
            foreign_column: foreign_column.to_column().qualify(name),
            alias: None,
        }
    }
    pub fn alias(mut self, alias: impl ToString) -> Self {
        self.alias = Some(alias.to_string());
        self.table_column = self.table_column.qualify(alias.to_string());
        self
    }
    pub fn replace_kind(mut self, kind: JoinKind) -> Self {
        self.kind = kind;
        self
    }
}

impl ToSql for Join {
    fn to_sql(&self) -> String {
        format!(
            r#"{} "{}"{} ON {} = {}"#,
            self.kind.to_string(),
            self.table_name.escape(),
            self.alias
                .as_ref()
                .map(|alias| format!(r#" AS "{}""#, alias.escape()))
                .unwrap_or(String::new()),
            self.table_column.to_sql(),
            self.foreign_column.to_sql(),
        )
    }
}

#[derive(
    Debug, Clone, Default, crate::prelude::Deserialize, crate::prelude::Serialize, PartialEq, Eq,
)]
pub struct Joins {
    joins: Vec<Join>,
}

impl Joins {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(mut self, join: Join) -> Self {
        self.joins.push(join);
        self
    }

    pub fn joins(&self) -> &[Join] {
        &self.joins
    }
}

impl ToSql for Joins {
    fn to_sql(&self) -> String {
        if self.joins.is_empty() {
            "".to_string()
        } else {
            format!(
                " {}",
                self.joins
                    .iter()
                    .map(|join| join.to_sql())
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        }
    }
}

#[derive(Debug, crate::prelude::Deserialize, crate::prelude::Serialize, Clone)]
pub struct Joined<S: Model, T: Model> {
    a: PhantomData<S>,
    b: PhantomData<T>,
    joins: Joins,
}

impl<S: Model, T: Model> PartialEq for Joined<S, T> {
    fn eq(&self, other: &Self) -> bool {
        self.joins.eq(&other.joins)
    }
}

impl<S: Model, T: Model> Eq for Joined<S, T> {}

impl<S: Model, T: Model> Joined<S, T> {
    pub fn new(join: Join) -> Self {
        Self {
            a: PhantomData,
            b: PhantomData,
            joins: Joins::new().add(join),
        }
    }

    pub fn join<U: Association<T>>(self) -> Joined<S, U> {
        let joins = self.joins.clone();
        let joins = joins.add(U::construct_join());
        Joined {
            a: PhantomData,
            b: PhantomData,
            joins,
        }
    }
    pub fn add_join<U: Model>(self, join: Join) -> Joined<S, U> {
        let joins = self.joins.clone().add(join);
        Joined {
            a: PhantomData,
            b: PhantomData,
            joins,
        }
    }
}

impl<S: Model, T: Model> From<Joined<S, T>> for Joins {
    fn from(val: Joined<S, T>) -> Self {
        val.joins
    }
}
