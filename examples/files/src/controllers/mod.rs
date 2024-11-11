use rwf::prelude::*;

#[derive(Default, macros::PageController)]
pub struct Upload;

#[async_trait]
impl PageController for Upload {
    async fn get(&self, _req: &Request) -> Result<Response, Error> {
        render!("templates/upload.html")
    }

    async fn post(&self, req: &Request) -> Result<Response, Error> {
        let form_data = req.form_data()?;
        if let Some(file) = form_data.file("file") {
            let redirect = format!("/ok?name={}&size={}", file.name, file.body.len());
            Ok(Response::new().redirect(redirect))
        } else {
            Ok(Response::bad_request())
        }
    }
}

#[derive(Default)]
pub struct UploadOk;

#[async_trait]
impl Controller for UploadOk {
    async fn handle(&self, req: &Request) -> Result<Response, Error> {
        let name = req.query().get_required::<String>("name")?;
        let size = req.query().get_required::<i64>("size")?;

        render!("templates/ok.html",
            "name" => rwf::http::urlencode(&name),
            "size" => size
        )
    }
}
