use rwf::prelude::*;

#[derive(rwf::macros::RestController, Default)]
pub struct MyController;

#[rwf::async_trait]
impl RestController for MyController {
    type Resource = i64; // Use integers as the resource identifiers.
                         // Can be any other data type that implements `rwf::controller::ToParameter` trait.

    /// GET /
    async fn list(&self, _request: &Request) -> Result<Response, Error> {
        let result = serde_json::json!([
            {"id": 5, "email": "test@test.com"},
            {"id": 7, "email": "hello@test.com"},
        ]);

        Ok(Response::new().json(result)?)
    }

    /// GET /:id
    async fn get(&self, _request: &Request, id: &Self::Resource) -> Result<Response, Error> {
        let result = serde_json::json!({
            "id": *id,
            "email": "guest@test.com",
        });

        Ok(Response::new().json(result)?)
    }

    // All other methods will return HTTP 501.
}
