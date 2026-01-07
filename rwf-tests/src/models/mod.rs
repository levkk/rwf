use rwf::async_trait;
use rwf::model::{Model, Scope};
use rwf::model::callbacks::Callback;
use rwf::prelude::{Deserialize, Serialize, ToResponse, ToSchema};
use rwf_macros::Model;

#[derive(Clone, Model, Debug, PartialEq, Serialize, Deserialize, ToSchema, ToResponse)]
#[has_many(Order)]
#[allow(dead_code)]
#[schema(title="An simple User.", description="A User/Customer implementation, which is referenced by Order")]
#[response(description="Representation of a single User azzoziated with the his Orders")]
pub struct User {
    #[schema(minimum=1, format="Int64", example=512)]
    pub(crate)id: Option<i64>,
    #[schema(required=true, examples("John", "Maria"))]
    pub(crate) name: String,
}

#[derive(Clone, Model, Debug, Serialize, Deserialize, ToSchema, ToResponse)]
#[belongs_to(Order)]
#[belongs_to(Product)]
#[allow(dead_code)]
pub struct OrderItem {
    #[schema(minimum=1, example=128, format="Int64")]
    pub(crate)id: Option<i64>,
    pub(crate)order_id: i64,
    pub(crate)product_id: i64,
    amount: f64,
}

#[derive(Clone, Model, Debug, Serialize, Deserialize, ToSchema, ToResponse)]
#[has_many(OrderItem)]
#[allow(dead_code)]
pub struct Product {
    pub(crate)id: Option<i64>,
    pub(crate) name: String,
    pub(crate) avg_price: f64,
}


#[derive(Debug, Default)]
pub struct CreateUserCallback;

#[async_trait]
impl Callback<User> for CreateUserCallback {
    async fn callback(mut self, data: User) -> User {
        eprintln!("{:?}", data);
        data
    }
}

#[derive(Clone, Model, Debug, Serialize, Deserialize, ToSchema, ToResponse)]
#[belongs_to(User)]
#[has_many(OrderItem)]
#[allow(dead_code)]
#[schema(title="An exaple Order", description="A Order in the DB System. Referebces the user who made the order and is refereenced by all related order items")]
#[response(description="Rerpresentation of a single Order azzoziated with the buying User and ordere3d Items")]
pub struct Order {
    #[schema(minimum=1, example=128, format="Int64")]
    pub(crate) id: Option<i64>,
    #[schema(minimum=1, example=32, format="Int64")]
    pub(crate) user_id: i64,
    pub(crate) name: String,
    #[schema(required=false, nullable=true)]
    pub(crate) optional: Option<String>,
}

impl OrderItem {
    pub fn expensive() -> Scope<Self> {
        Self::all().filter_gt("amount", 5.0)
    }
}

pub mod oapi_backend {
    use super::{User, Order, OrderItem, Product};
    use rwf::prelude::*;
    use rwf::http::{Request, Response};
    use rwf::controller::ModelListQuery;

    #[utoipa::path(
        get,
        path = "/orders",
        responses(
            (status = 200, body=Vec<Order>),
            (status = 500, description="Server Error")
        ),
        params(
            ModelListQuery
        )
    )]
    fn list_orders(_request: &Request) -> Result<Response, rwf::http::Error> {
        Ok(Response::not_implemented())
    }

    #[utoipa::path(
        post,
        path = "/orders",
        responses(
            (status = 200, body=Order),
            (status = 400, description = "Invalid User Input"),
            (status = 500, description="Server Error")),
        request_body(content= Order, description = "The new Model to create")
    )]
    fn create_order(_request: &Request) -> Result<Response, rwf::http::Error> {
        Ok(Response::not_implemented())
    }

    #[utoipa::path(get, path = "/orders/{id}", responses((status = 200, body=Order), (status = 404, description = "No such model found"), (status = 500, description = "Server Error")
    ), params(("id" = i64, Path, description = "Database ID of the Model")))]
    fn get_order(_request: &Request) -> Result<Response, rwf::http::Error> {
        Ok(Response::not_implemented())
    }

    #[utoipa::path(put, path = "/orders/{id}", responses((status = 200, body=Order), (status = 400, description = "Invalid User Input"), (status = 404, description = "No such model found"), (status = 500, description = "Server Error")
    ), params(("id" = i64, Path, description = "Database ID of the Model")), request_body(content=Order, description="Full Model for full update"
    ))]
    fn update_order(_request: &Request) -> Result<Response, rwf::http::Error> {
        Ok(Response::not_implemented())
    }

    #[utoipa::path(patch, path = "/orders/{id}", responses((status = 200, body=Order), (status = 400, description="Invalid User Input"),(status = 404, description = "No such model found"), (status = 500, description = "Server Error")
    ), params(("id" = i64, Path, description = "Database ID of the Model")), request_body(
        content_type = "application/json",
        description = "Partial Model for partial update"
    ))]
    fn patch_order(_request: &Request) -> Result<Response, rwf::http::Error> {
        Ok(Response::not_implemented())
    }

    #[utoipa::path(delete, path = "/orders/{id}", responses((status = 200, body=Order), (status = 404, description = "No such model found"), (status = 500, description = "Server Error")
    ), params(("id" = i64, Path, description = "Database ID of the Model")))]
    fn delete_order(_request: &Request) -> Result<Response, rwf::http::Error> {
        Ok(Response::not_implemented())
    }

    #[derive(OpenApi)]
    #[openapi(
        components(
            schemas(User, Order, Product, OrderItem),
            responses(Order),
        ),
        paths(list_orders, create_order, get_order, delete_order, update_order, patch_order)
    )]
    pub struct OpenApiDocs;
}