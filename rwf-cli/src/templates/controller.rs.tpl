use rwf::prelude::*;

#[derive(Default)]
pub struct <%= name %>;

#[async_trait]
impl Controller for <%= name %> {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::not_implemented())
    }
}
