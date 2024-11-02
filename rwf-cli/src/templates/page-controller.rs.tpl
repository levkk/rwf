use rwf::prelude::*;

#[derive(Default, macros::PageController)]
pub struct <%= name %>;

#[async_trait]
impl PageController for <%= name %> {
    async fn get(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::not_implemented())
    }

    async fn post(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::method_not_allowed())
    }
}
