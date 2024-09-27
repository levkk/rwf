use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    #[serde(rename = "u")]
    user_id: i64,
    #[serde(rename = "e")]
    expires: u64, // unix timestamp
}