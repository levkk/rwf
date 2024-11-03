use crate::models::RequestByCode;
use rwf::prelude::*;

#[derive(Default)]
pub struct Requests;

#[async_trait]
impl Controller for Requests {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        let requests = {
            let mut conn = Pool::connection().await?;
            RequestByCode::count(60).fetch_all(&mut conn).await?
        };
        let requests = serde_json::to_string(&requests)?;

        render!("templates/rwf_admin/requests.html", "requests" => requests)
    }
}
