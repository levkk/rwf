//! Implements the `DELETE` statement.
use super::{
    Association, Column, Escape, FromRow, Model, Placeholders, Select, ToColumn, ToSql, ToValue,
    WhereClause,
};
use crate::model::select::FilterQuery;
use crate::model::temporary::{With, WithQuery};
use std::marker::PhantomData;

#[derive(Debug, Clone, crate::prelude::Deserialize, crate::prelude::Serialize)]
pub struct Delete<T> {
    table_name: String,
    primary_key: String,
    where_clause: WhereClause,
    pub placeholders: Placeholders,
    marker: PhantomData<T>,
    with: With,
    using: Vec<String>,
}

impl<T: Model> Delete<T> {
    pub fn empty() -> Self {
        Self {
            table_name: T::table_name().to_string(),
            primary_key: T::primary_key().to_string(),
            where_clause: WhereClause::default(),
            placeholders: Placeholders::default(),
            marker: PhantomData,
            with: With::default(),
            using: vec![],
        }
    }

    /// Use another Table to select the entries that will be deleted
    /// # Example
    /// ```
    /// use aes::cipher::typenum::Or;
    /// use time::{Date, Month};
    /// use rwf::model::{Column, Delete, Placeholders};
    /// use rwf::model::prelude::*;
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// struct Order {
    ///     id: Option<i64>,
    ///     user_id: i64,
    ///     date: Date
    /// }
    /// let delete: Delete<Order> = Delete::empty().using("users").filter_and("user_id", Column::new("users", "id")).filter_lt("date", Date::from_calendar_date(2024, Month::January, 1).unwrap());
    /// assert_eq!(
    ///     delete.to_sql(),
    ///     r#"DELETE FROM "orders" USING "users" WHERE "orders"."user_id" = "users"."id" AND "orders"."date" < $1 RETURNING *"#
    /// );
    /// assert_eq!(
    ///     delete.placeholders,
    ///     Placeholders::from(vec![Date::from_calendar_date(2024, Month::January, 1).unwrap().to_value()])
    /// );
    /// ```
    pub fn using(mut self, using: impl ToString) -> Self {
        self.using.push(using.to_string());
        self
    }

    /// Use a Foreign Key Constraint as Condition for a `Deltete` Query
    /// # Example
    /// ```
    /// use rwf::model::{Model, Query, ToSql, Delete};
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// #[has_many(Order)]
    /// struct User {
    ///     id: Option<i64>
    /// }
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// #[has_many(OrderItem)]
    /// #[belongs_to(User)]
    /// struct Order {
    ///     id: Option<i64>,
    ///     user_id: i64
    /// }
    /// #[derive(Clone, rwf::prelude::Serialize, rwf::prelude::Deserialize, rwf::macros::Model)]
    /// #[belongs_to(Order)]
    /// struct OrderItem {
    ///     id: Option<i64>,
    ///     order_id: i64
    /// }
    /// assert_eq!(
    ///     Delete::empty().using_join::<User>().to_sql(),
    ///     r#"DELETE FROM "orders" USING "users" WHERE "orders"."user_id" = "users"."id" RETURNING *"#
    /// );
    /// assert_eq!(
    ///     Delete::empty().using_join::<OrderItem>().to_sql(),
    ///     r#"DELETE FROM "orders" USING "order_items" WHERE "orders"."id" = "order_items"."order_id" RETURNING *"#
    /// );
    ///
    /// ```
    pub fn using_join<F: Association<T>>(self) -> Self {
        if F::belongs_to() {
            self.filter_and(
                T::primary_key(),
                T::foreign_key()
                    .to_column()
                    .qualify(F::table_name())
                    .to_value(),
            )
        } else {
            self.filter_and(
                F::foreign_key(),
                F::primary_key()
                    .to_column()
                    .qualify(F::table_name())
                    .to_value(),
            )
        }
        .using(F::table_name())
    }
}

impl<T: Model, C: ToColumn, V: ToValue> FromIterator<(C, V)> for Delete<T> {
    fn from_iter<I: for<'a> IntoIterator<Item = (C, V)>>(iter: I) -> Self {
        let query = Self::empty();
        iter.into_iter()
            .fold(query, |q, (c, v)| q.filter_and(c, v.to_value()))
    }
}

impl<T: Model, C: ToColumn, V: ToValue> From<&[(C, V)]> for Delete<T> {
    fn from(v: &[(C, V)]) -> Self {
        Self::from_iter(
            v.iter()
                .map(|(c, v)| (c.to_column(), v.to_value()))
                .collect::<Vec<_>>(),
        )
    }
}

impl<T: Model> From<Select<T>> for Delete<T> {
    fn from(select: Select<T>) -> Delete<T> {
        let mut delete = Delete::empty();
        delete.where_clause = select.where_clause;
        delete.placeholders = select.placeholders;
        delete.with = select.with;
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
        let using = if self.using.is_empty() {
            String::new()
        } else {
            format!(
                " USING {}",
                self.using
                    .iter()
                    .map(|s| format!(r#""{}""#, s.escape()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        format!(
            r#"{}DELETE FROM "{}"{}{} RETURNING *"#,
            self.with.to_sql(),
            self.table_name.escape(),
            using,
            self.where_clause.to_sql(),
        )
    }
}

impl<T: FromRow> FilterQuery for Delete<T> {
    fn get_table_name(&self) -> &str {
        self.table_name.as_str()
    }

    fn get_where_clause(&self) -> &WhereClause {
        &self.where_clause
    }

    fn get_placeholders(&self) -> &Placeholders {
        &self.placeholders
    }

    fn get_where_clause_mut(&mut self) -> &mut WhereClause {
        &mut self.where_clause
    }

    fn get_placeholders_mut(&mut self) -> &mut Placeholders {
        &mut self.placeholders
    }
}

impl<T: FromRow> WithQuery for Delete<T> {
    fn with_statements(&self) -> &With {
        &self.with
    }

    fn with_statements_mut(&mut self) -> &mut With {
        &mut self.with
    }

    fn get_statement_offset(&self) -> i32 {
        self.where_clause.placeholders() as i32
    }

    fn add_offset(&mut self, offset: i32) {
        self.where_clause.add_offset(offset);
    }
    fn placeholders(&self) -> Placeholders {
        let mut placeholders = self.with.placeholders();
        placeholders.push(self.placeholders.clone());
        Placeholders::from_iter(placeholders)
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
    #[test]
    fn test_from_iter() {
        let delete = Delete::<User>::from([("name", "John")].as_slice());
        assert_eq!(
            delete.to_sql(),
            r#"DELETE FROM "users" WHERE "users"."name" = $1 RETURNING *"#
        );
        assert_eq!(
            delete.get_placeholders(),
            &Placeholders::from(vec!["John".to_value()])
        );
    }
}
