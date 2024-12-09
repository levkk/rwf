use rwf::prelude::*;

#[derive(Clone, macros::Model, macros::UserModel, Debug)]
#[table_name("rwf_auth_users")]
pub struct User {
    id: Option<i64>,
    identifier: String,
    password: String,
    created_at: OffsetDateTime,
}
