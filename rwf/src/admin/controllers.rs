use crate::prelude::*;

#[derive(Default)]
pub struct Index;

#[async_trait]
impl Controller for Index {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        let template = Template::load("templates/rwf_admin/index.html")?;
        Ok(Response::new().html(template.render_default()?))
    }
}

#[derive(Default)]
pub struct Jobs;

#[async_trait]
impl Controller for Jobs {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        let template = Template::load("templates/rwf_admin/jobs.html")?;
        Ok(Response::new().html(template.render_default()?))
    }
}
