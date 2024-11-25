use crate::models::{RequestByCode, RequestsDuration};
use rwf::prelude::*;

#[derive(Default)]
pub struct Requests;

#[async_trait]
impl Controller for Requests {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let requests = {
            let mut conn = Pool::connection().await?;
            RequestByCode::count(60).fetch_all(&mut conn).await?
        };

        let duration = {
            let mut conn = Pool::connection().await?;
            RequestsDuration::count(60).fetch_all(&mut conn).await?
        };

        let requests = serde_json::to_string(&requests)?;
        let duration = serde_json::to_string(&duration)?;

        render!(request, "templates/rwf_admin/requests.html",
            "title" => "Requests | Rust Web Framework",
            "requests" => requests,
            "duration" => duration,
        )
    }
}
