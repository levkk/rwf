//! A chat user.

use time::OffsetDateTime;

#[derive(Clone, rwf::macros::Model)]
pub struct User {
    pub id: Option<i64>,
    pub name: String,
    pub created_at: OffsetDateTime,
}
