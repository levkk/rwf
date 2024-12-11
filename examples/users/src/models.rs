use rwf::prelude::*;

#[derive(Clone, macros::Model, macros::UserModel)]
#[user_model(email, password)]
pub struct User {
    id: Option<i64>,
    email: String,
    password: String,
    created_at: OffsetDateTime,
}
