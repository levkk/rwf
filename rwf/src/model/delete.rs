//! Implements the `DELETE` statement.
use super::{Column, Escape, FromRow, Model, Placeholders, Select, ToSql, WhereClause};
use std::marker::PhantomData;

#[derive(Debug, Clone, crate::prelude::Deserialize, crate::prelude::Serialize)]
pub struct Delete<T> {
    table_name: String,
    primary_key: String,
    where_clause: WhereClause,
    pub placeholders: Placeholders,
    marker: PhantomData<T>,
}

impl<T: Model> Delete<T> {
    pub fn empty() -> Self {
        Self {
            table_name: T::table_name().to_string(),
            primary_key: T::primary_key().to_string(),
            where_clause: WhereClause::default(),
            placeholders: Placeholders::default(),
            marker: PhantomData,
        }
    }
}

impl<T: Model> From<Select<T>> for Delete<T> {
    fn from(select: Select<T>) -> Delete<T> {
        let mut delete = Delete::empty();
        delete.where_clause = select.where_clause;
        delete.placeholders = select.placeholders;

        delete
    }
}

impl<T: Model> From<T> for Delete<T> {
    fn from(t: T) -> Self {
        let mut delete = Delete::empty();
        let column = Column::new(&delete.table_name, &delete.primary_key);
        delete.where_clause.add(column, t.id());
        delete
    }
}

impl<T: FromRow> ToSql for Delete<T> {
    fn to_sql(&self) -> String {
        format!(
            r#"DELETE FROM "{}"{} RETURNING *"#,
            self.table_name.escape(),
            self.where_clause.to_sql(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Error, FromRow, Model, ToSql, ToValue, Value};
    use crate::prelude::Deserialize;
    use tokio_postgres::Row;

    #[derive(Debug, Deserialize, Clone)]
    struct User {
        id: Option<i64>,
        name: String,
    }
    impl FromRow for User {
        fn from_row(row: Row) -> Result<Self, Error>
        where
            Self: Sized,
        {
            Ok(Self {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
            })
        }
    }
    impl Model for User {
        fn table_name() -> &'static str {
            "users"
        }

        fn column_names() -> &'static [&'static str] {
            &["name"]
        }

        fn id(&self) -> Value {
            self.id.to_value()
        }

        fn values(&self) -> Vec<Value> {
            vec![self.name.to_value()]
        }

        fn foreign_key() -> &'static str {
            "user_id"
        }
    }

    #[test]
    fn test_from_model() {
        let user = User {
            id: Some(3),
            name: "username".to_string(),
        };
        let query = Delete::from(user);
        assert_eq!(
            query.to_sql(),
            r#"DELETE FROM "users" WHERE "users"."id" = 3 RETURNING *"#
        )
    }
}
