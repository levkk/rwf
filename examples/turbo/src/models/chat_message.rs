//! Chat message.
use time::OffsetDateTime;

#[derive(Clone, rwf::macros::Model)]
pub struct ChatMessage {
    id: Option<i64>,
    user_id: i64,
    body: String,
    created_at: OffsetDateTime,
}
