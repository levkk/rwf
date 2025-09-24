use rwf::prelude::*;

#[derive(Clone, macros::Model, Debug, macros::UserModel)]
#[table_name("rwf_auth_users")]
#[user_model(identifier, password)]
pub struct User {
    id: Option<i64>,
    identifier: String,
    password: String,
    created_at: OffsetDateTime,
}
