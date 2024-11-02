use rwf::prelude::*;

#[derive(Default)]
pub struct Index;

#[async_trait]
impl Controller for Index {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Response::new().redirect("jobs"))
    }
}
