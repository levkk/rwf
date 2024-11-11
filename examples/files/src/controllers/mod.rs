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
            render!("templates/ok.html",
                "name" => file.name(),
                "size" => file.body().len() as i64,
                "content_type" => file.content_type(),
                "comment" => comment,
            201); // 201 = created
        } else {
            Ok(Response::bad_request())
        }
    }
}
