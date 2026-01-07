pub mod login;

#[cfg(test)]
mod tests {
    use crate::models::oapi_backend::OpenApiDocs;
    use crate::{models, OpenApiController};
    use serde_json::{json, to_string_pretty};
    use utoipa::OpenApi;
    use utoipa_rapidoc::RapiDoc;
    use utoipa_swagger_ui::Config;

    #[test]
    fn test_redoc() {
        use utoipa_redoc::Redoc;
        let html_empty = utoipa_redoc::Redoc::new(json!({"openapi": "3.1.ß"}));
        let redoc = utoipa_redoc::Redoc::new(OpenApiDocs::openapi());
        assert_ne!(html_empty.to_html(), redoc.to_html());
        assert!(redoc.to_html().len() > html_empty.to_html().len());
    }
    #[test]
    fn test_swagger() {
        use utoipa_rapidoc::RapiDoc;
        let rapidoc = RapiDoc::new("./rapidoc.json");
        let html = rapidoc.to_html();
        let rwfcli: utoipa::openapi::OpenApi;
    }
}
