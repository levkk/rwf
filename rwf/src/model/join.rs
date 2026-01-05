//! Implements joining tables in a `SELECT` query.
use super::{Column, Escape, Model, ToSql};
use std::marker::PhantomData;

/// Type of relationship between models.
#[derive(PartialEq)]
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
                }
            }

            // INNER JOIN "orders ON "orders"."user_id" = "users"."id"
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
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone, crate::prelude::Deserialize, crate::prelude::Serialize)]
pub enum JoinKind {
    Inner,
    Left,
    Outer,
}

impl ToString for JoinKind {
    fn to_string(&self) -> String {
        match self {
            JoinKind::Inner => "INNER JOIN",
            JoinKind::Left => "LEFT JOIN",
            JoinKind::Outer => "OUTER JOIN",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, crate::prelude::Deserialize, crate::prelude::Serialize)]
pub struct Join {
    kind: JoinKind,
    table_name: String,
    table_column: Column,
    foreign_column: Column,
}

impl Join {
    fn replace_kind(mut self, kind: JoinKind) -> Self {
        self.kind = kind;
        self
    }
}

impl ToSql for Join {
    fn to_sql(&self) -> String {
        format!(
            r#"{} "{}" ON {} = {}"#,
            self.kind.to_string(),
            self.table_name.escape(),
            self.table_column.to_sql(),
            self.foreign_column.to_sql(),
        )
    }
}

#[derive(Debug, Clone, Default, crate::prelude::Deserialize, crate::prelude::Serialize)]
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

#[derive(Debug, crate::prelude::Deserialize, crate::prelude::Serialize)]
pub struct Joined<S: Model, T: Model> {
    a: PhantomData<S>,
    b: PhantomData<T>,
    joins: Joins,
}

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
}

impl<S: Model, T: Model> Into<Joins> for Joined<S, T> {
    fn into(self) -> Joins {
        self.joins
    }
}
