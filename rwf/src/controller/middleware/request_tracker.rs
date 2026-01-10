//! Record HTTP requests served by the application.
//!
//! Requests record metadata like client IP, request duration, path, query, and HTTP method.
//! Each client is given a cookie which uniquely identifies that browser. This allows to record unique sessions.
//!
//! You can view requests in real time in the [admin panel](https://github.com/levkk/rwf/tree/main/rwf-admin), or by querying the `rwf_requests` table, e.g.:
//!
//! ```sql
//! SELECT * FROM rwf_requests
//! WHERE created_at > NOW() - INTERVAL '5 minutes';
//! ```
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use utoipa::openapi::OpenApi;
use uuid::Uuid;

use crate::analytics::Request as AnalyticsRequest;
use crate::controller::middleware::prelude::*;
use crate::http::CookieBuilder;
use crate::model::{Model, Pool, ToValue};

static COOKIE_NAME: &str = "rwf_aid";
static COOKIE_DURATION: Duration = Duration::days(399);

#[derive(Serialize, Deserialize)]
struct AnalyticsCookie {
    #[serde(rename = "u")]
    uuid: String,
    #[serde(rename = "e")]
    expires: i64,
}

impl AnalyticsCookie {
    fn uuid(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.uuid).ok()
    }

    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            expires: (OffsetDateTime::now_utc() + COOKIE_DURATION).unix_timestamp(),
        }
    }

    fn should_renew(&self) -> bool {
        match OffsetDateTime::from_unix_timestamp(self.expires) {
            Ok(timestamp) => timestamp - OffsetDateTime::now_utc() < Duration::days(7),
            Err(_) => true,
        }
    }

    fn to_network(&self) -> String {
        let json = serde_json::to_string(self).unwrap();
        general_purpose::STANDARD_NO_PAD.encode(&json)
    }

    fn from_network(s: &str) -> Option<Self> {
        match general_purpose::STANDARD_NO_PAD.decode(s) {
            Ok(v) => serde_json::from_slice::<Self>(&v).ok(),

            Err(_) => None,
        }
    }
}

/// HTTP request tracker.
pub struct RequestTracker {}

impl RequestTracker {
    /// Creates new HTTP request tracker.
    pub fn new() -> Self {
        Self {}
    }
}

#[crate::async_trait]
impl Middleware for RequestTracker {
    async fn handle_request(&self, request: Request) -> Result<Outcome, Error> {
        Ok(Outcome::Forward(request))
    }

    async fn handle_response(
        &self,
        request: &Request,
        mut response: Response,
    ) -> Result<Response, Error> {
        let method = request.method().to_string();
        let path = request.path().path().to_string();
        let query = request.path().query().to_json();
        let code = response.status().code() as i32;
        let duration =
            ((OffsetDateTime::now_utc() - request.received_at()).as_seconds_f64() * 1000.0) as f32;
        let client = request.peer().ip();

        let (create, cookie) = match request
            .cookies()
            .get(COOKIE_NAME)
            .map(|cookie| AnalyticsCookie::from_network(cookie.value()))
        {
            Some(Some(cookie)) => (cookie.should_renew(), cookie),
            _ => (true, AnalyticsCookie::new()),
        };

        if create {
            let cookie = CookieBuilder::new()
                .name(COOKIE_NAME)
                .value(cookie.to_network())
                .max_age(Duration::days(399))
                .build();

            response = response.cookie(cookie);
        }

        if let Ok(mut conn) = Pool::connection().await {
            if let Some(client_id) = cookie.uuid() {
                let _ = AnalyticsRequest::create(&[
                    ("method", method.to_value()),
                    ("path", path.to_value()),
                    ("query", query.to_value()),
                    ("client_ip", client.to_value()),
                    ("client_id", client_id.to_value()),
                    ("code", code.to_value()),
                    ("duration", duration.to_value()),
                ])
                .execute(&mut conn)
                .await;
            }
        }

        Ok(response)
    }
}

impl utoipa::Modify for RequestTracker {
    fn modify(&self, _openapi: &mut OpenApi) {}
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_request_tracker() {
        let request = Request::default();
        let response = Response::default();

        let mut response = RequestTracker::new()
            .handle_response(&request, response)
            .await
            .unwrap();
        assert!(response.cookies().get(COOKIE_NAME).is_some());
    }
}
