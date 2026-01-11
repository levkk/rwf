use rwf::controller::middleware::SecureId;
use rwf::controller::{Middleware, MiddlewareSet, ModelController, OpenApiController};
use rwf::macros::generate_openapi_model_controller;
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

#[tokio::main]
async fn main() -> Result<(), rwf::http::Error> {
    // Create a handler to make the specs available
    //let apictrl = crud!("/product" => ProductController);
    // Create a handler without the overwrite of the id field

    let apictrl = ProductController {
        middleware: MiddlewareSet::default(),
    }
    .crud("/prod");

    // Makes the OpenApi Specs available under /openapi/yaml or /openapi/json
    // Serve an API Browser under /openapi/redoc and /openapi/rapidoc
    let openapi = route!("/openapi" => OpenApiController);

    let routes = vec![apictrl, openapi];
    rwf::http::Server::new(routes).launch().await
}
