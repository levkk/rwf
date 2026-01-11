use rwf::controller::middleware::SecureId;
use rwf::controller::{Middleware, MiddlewareSet, ModelController, OpenApiController};
use rwf::macros::{generate_openapi_model_controller, generate_openapi_specs};
use rwf::model::migrate;
use rwf::prelude::*;

/// A Model that can be used in the OpenApi Spec
#[derive(Debug, Serialize, Deserialize, Clone, macros::Model, ToSchema, ToResponse)]
#[schema(description = "Product Representation with available Stock and Price", examples(json!({"id": 1, "name": "Test Product", "price": 4.50, "stock": 1000})))]
#[response(description = "A possible Product Response", examples(("ExampleProduct" = (summary="Example Response Product", value=json!({"id": 1, "name": "Test Product", "price": 4.50, "stock": 1000})))))]
struct Product {
    #[schema(format = "Int64", example = 1024, minimum = 1)]
    id: Option<i64>,
    #[schema(example = "Product Name")]
    name: String,
    #[schema(example = 5.99, minimum = 0.01, multiple_of = 0.01, format = "Double")]
    price: f64,
    #[schema(example = 1000, minimum = 0, format = "Int64")]
    stock: i64,
}
/// Generate the ModelController implementation
/// Overwrite the ApiSpecs so that no real Id Field is shown, but an encrypted
#[generate_openapi_model_controller(i64, Product)]
#[derive(macros::ModelController)]
#[middleware(middleware)]
struct ProductController {
    middleware: MiddlewareSet,
}

impl Default for ProductController {
    fn default() -> Self {
        Self {
            middleware: MiddlewareSet::new(vec![SecureId::default().middleware()]),
        }
    }
}

#[derive(macros::Form, macros::TemplateValue, Clone)]
struct PageForm {
    data: String,
}

#[derive(Default, macros::PageController)]
struct PageFormController;

/// Try to generate OpenApi Specs for non ModelController
#[generate_openapi_specs]
#[async_trait]
impl PageController for PageFormController {
    async fn get(&self, request: &Request) -> Result<Response, Error> {
        render!(request, "templates/page.html")
    }
    async fn post(&self, request: &Request) -> Result<Response, Error> {
        let data: PageForm = request.form()?;
        let stream = turbo_stream!(request, "templates/page_element.html", "list", "elem" => data)
            .action("append");
        Ok(Response::new().turbo_stream(&[stream]))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ToResponse)]
struct Body {
    data: String,
    ts: OffsetDateTime,
}
#[derive(Default)]
struct BodyController;

#[generate_openapi_specs(Body)]
#[async_trait]
impl Controller for BodyController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let body = Body {
            data: serde_json::to_string(&request.head())?,
            ts: OffsetDateTime::now_utc(),
        };
        Ok(Response::new().json(body)?)
    }
}

#[tokio::main]
async fn main() -> Result<(), rwf::http::Error> {
    migrate().await?;

    // Create a handler to make the specs available
    let apictrl = crud!("/product" => ProductController);

    // Create a handler without the overwrite of the id field
    //let apictrl = ProductController {middleware: MiddlewareSet::default()}.crud("/prod");

    // Generate PageController with ApiSpecs for get and post Mehthods
    let page = route!("/page" => PageFormController);
    // Makes the OpenApi Specs available under /openapi/yaml or /openapi/json
    // Serve an API Browser under /openapi/redoc and /openapi/rapidoc
    let openapi = route!("/openapi" => OpenApiController);

    // Generate a Controller with ApiSpecs and a hint for the Json Type!
    let bodyctrl = route!("/body" => BodyController);

    let routes = vec![
        apictrl,
        page,
        openapi,
        bodyctrl,
        route!("/turbo_stream" => rwf::controller::TurboStream),
    ];
    rwf::http::Server::new(routes).launch().await
}
