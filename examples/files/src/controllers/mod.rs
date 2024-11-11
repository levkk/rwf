use rwf::http::urlencode;
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
        let comment = form_data.get_required::<String>("comment")?;
        if let Some(file) = form_data.file("file") {
            let redirect = format!(
                "/ok?name={}&size={}&content_type={}&comment={}",
                urlencode(file.name()),
                file.body().len(),
                urlencode(file.content_type()),
                urlencode(&comment),
            );
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
        let content_type = req.query().get_required::<String>("content_type")?;
        let comment = req.query().get_required::<String>("comment")?;

        render!("templates/ok.html",
            "name" => rwf::http::urlencode(&name),
            "size" => size,
            "content_type" => content_type,
            "comment" => comment,
        )
    }
}
