use super::{Column, Escape, Model, ToSql};

pub enum AssociationType {
    BelongsTo,
    HasMany,
    HasOne,
}

pub trait Association<T: Model>: Model {
    fn association_type() -> AssociationType {
        AssociationType::BelongsTo
    }

    fn join() -> Join {
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
