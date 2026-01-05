//! Chat message.
use super::User;
use time::OffsetDateTime;

#[derive(Clone, rwf::macros::Model, rwf::prelude::Serialize, rwf::prelude::Deserialize)]
#[belongs_to(User)]
pub struct ChatMessage {
    pub id: Option<i64>,
    pub user_id: i64,
    pub body: String,
    pub created_at: OffsetDateTime,
}
