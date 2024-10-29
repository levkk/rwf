use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::analytics::Request as AnalyticsRequest;
use crate::controller::middleware::prelude::*;
use crate::http::CookieBuilder;
use crate::model::{Model, Pool, ToValue};

static COOKIE_NAME: &str = "rwf_aid";

pub struct RequestTracker {}

impl RequestTracker {
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
            ((request.received_at() - OffsetDateTime::now_utc()).as_seconds_f64() * 1000.0) as f32;
        let client = request.peer().ip();

        let client_id = match request
            .cookies()
            .get(COOKIE_NAME)
            .map(|cookie| Uuid::parse_str(cookie.value()))
        {
            Some(Ok(cookie)) => cookie,
            _ => Uuid::new_v4(),
        };

        let cookie = CookieBuilder::new()
            .name(COOKIE_NAME)
            .value(client_id.to_string())
            .max_age(Duration::weeks(4))
            .build();

        response = response.cookie(cookie);

        if let Ok(mut conn) = Pool::connection().await {
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

        Ok(response)
    }
}
