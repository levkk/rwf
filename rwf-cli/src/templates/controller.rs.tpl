use rwf::prelude::*;

#[derive(Default)]
pub struct <%= name %>;

#[async_trait]
impl Controller for <%= name %> {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Response::not_implemented())
    }
}
