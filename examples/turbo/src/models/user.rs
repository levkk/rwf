//! A chat user.

use super::ChatMessage;
use time::OffsetDateTime;

#[derive(Clone, rwf::macros::Model)]
#[has_many(ChatMessage)]
pub struct User {
    pub id: Option<i64>,
    pub name: String,
    pub created_at: OffsetDateTime,
}
