use std::net::IpAddr;

use crate::model::{Error, FromRow, Model, ToValue, Value};

use time::OffsetDateTime;

#[derive(Clone)]
pub struct Request {
    id: Option<i64>,
    path: String,
    method: String,
    query: serde_json::Value,
    code: i32,
    client_ip: Option<IpAddr>,
    created_at: OffsetDateTime,
    duration: f32,
}

impl FromRow for Request {
    fn from_row(row: tokio_postgres::Row) -> Result<Self, Error> {
        Ok(Self {
            id: row.try_get("id")?,
            path: row.try_get("path")?,
            method: row.try_get("method")?,
            query: row.try_get("query")?,
            code: row.try_get("code")?,
            client_ip: row.try_get("client")?,
            created_at: row.try_get("created_at")?,
            duration: row.try_get("duration")?,
        })
    }
}

impl Model for Request {
    fn id(&self) -> Value {
        self.id.to_value()
    }

    fn table_name() -> &'static str {
        "rwf_requests"
    }

    fn foreign_key() -> &'static str {
        "rwf_request_id"
    }

    fn column_names() -> &'static [&'static str] {
        &[
            "path",
            "method",
            "query",
            "code",
            "client_ip",
            "created_at",
            "duration",
        ]
    }

    fn values(&self) -> Vec<Value> {
        vec![
            self.path.to_value(),
            self.method.to_value(),
            self.query.to_value(),
            self.code.to_value(),
            self.client_ip.to_value(),
            self.created_at.to_value(),
            self.duration.to_value(),
        ]
    }
}
