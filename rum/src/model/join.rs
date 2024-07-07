use super::{Column, Escape, Model, Query, ToSql};
use std::marker::PhantomData;

#[derive(PartialEq)]
pub enum AssociationType {
    BelongsTo,
    HasMany,
    HasOne,
}

#[derive(PartialEq)]
pub struct BelongsTo<S: Model, T: Model> {
    owner: PhantomData<S>,
    target: PhantomData<T>,
}

impl<S: Model, T: Model> BelongsTo<S, T> {
    pub fn join() -> Join {
        let table_name = S::table_name();
        let table_column = Column::new(T::table_name(), T::primary_key());
        let foreign_column = Column::new(S::table_name(), T::foreign_key());
        Join {
            kind: JoinKind::Inner,
            table_name,
            table_column,
            foreign_column,
        }
    }
}

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

    fn construct_join() -> Join {
        use AssociationType::*;

        match Self::association_type() {
            // INNER JOIN "users" ON "users"."id" = "orders"."user_id"
            BelongsTo => {
                let table_name = Self::table_name();
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
            HasMany => {
                let table_name = Self::table_name();
                let table_column = Column::new(T::table_name(), Self::foreign_key());
                let foreign_column = Column::new(Self::table_name(), Self::primary_key());
                Join {
                    kind: JoinKind::Inner,
                    table_name,
                    table_column,
                    foreign_column,
                }
            }

            _ => todo!(),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
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

#[derive(Debug, Clone)]
pub struct Join {
    kind: JoinKind,
    table_name: String,
    table_column: Column,
    foreign_column: Column,
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

#[derive(Debug, Clone, Default)]
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

#[derive(Debug)]
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

    pub fn join<U: Association<T>>(mut self) -> Joined<S, U> {
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
