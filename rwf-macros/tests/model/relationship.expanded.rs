use rwf_macros::*;
#[has_many(Task)]
pub struct User {
    id: Option<i64>,
    email: String,
}
#[automatically_derived]
impl rwf::model::FromRow for User {
    fn from_row(row: rwf::tokio_postgres::Row) -> Result<Self, rwf::model::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            email: row.try_get("email")?,
        })
    }
}
#[automatically_derived]
impl rwf::model::Model for User {
    fn table_name() -> &'static str {
        "users"
    }
    fn foreign_key() -> &'static str {
        "user_id"
    }
    fn column_names() -> &'static [&'static str] {
        &["email"]
    }
    fn values(&self) -> Vec<rwf::model::Value> {
        use rwf::model::ToValue;
        <[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new([self.email.to_value()]))
    }
    fn id(&self) -> rwf::model::Value {
        use rwf::model::ToValue;
        self.id.to_value()
    }
}
#[automatically_derived]
impl rwf::model::Association<Task> for User {
    fn association_type() -> rwf::model::AssociationType {
        rwf::model::AssociationType::HasMany
    }
}
#[automatically_derived]
impl ::core::clone::Clone for User {
    #[inline]
    fn clone(&self) -> User {
        User {
            id: ::core::clone::Clone::clone(&self.id),
            email: ::core::clone::Clone::clone(&self.email),
        }
    }
}
#[belongs_to(User)]
pub struct Task {
    id: Option<i64>,
    user_id: i64,
}
#[automatically_derived]
impl rwf::model::FromRow for Task {
    fn from_row(row: rwf::tokio_postgres::Row) -> Result<Self, rwf::model::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
        })
    }
}
#[automatically_derived]
impl rwf::model::Model for Task {
    fn table_name() -> &'static str {
        "tasks"
    }
    fn foreign_key() -> &'static str {
        "task_id"
    }
    fn column_names() -> &'static [&'static str] {
        &["user_id"]
    }
    fn values(&self) -> Vec<rwf::model::Value> {
        use rwf::model::ToValue;
        <[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new([self.user_id.to_value()]))
    }
    fn id(&self) -> rwf::model::Value {
        use rwf::model::ToValue;
        self.id.to_value()
    }
}
#[automatically_derived]
impl rwf::model::Association<User> for Task {
    fn association_type() -> rwf::model::AssociationType {
        rwf::model::AssociationType::BelongsTo
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Task {
    #[inline]
    fn clone(&self) -> Task {
        Task {
            id: ::core::clone::Clone::clone(&self.id),
            user_id: ::core::clone::Clone::clone(&self.user_id),
        }
    }
}
fn main() {}
