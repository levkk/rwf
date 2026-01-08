use rwf::async_trait;
use rwf::model::callbacks::Callback;
use rwf::model::{Model, Scope};
use rwf::prelude::{utoipa, Deserialize, OpenApi, Serialize, ToResponse, ToSchema};
use rwf_macros::Model;

#[rwf_macros::generate_full_model(i64, UserController, users)]
#[derive(Clone, Model, Debug, PartialEq, Serialize, Deserialize, ToSchema, ToResponse)]
#[has_many(Order)]
#[allow(dead_code)]
#[schema(
    title = "An simple User.",
    description = "A User/Customer implementation, which is referenced by Order"
)]
#[response(description = "Representation of a single User azzoziated with the his Orders")]
pub struct User {
    #[schema(minimum = 1, format = "Int64", example = 512)]
    pub(crate) id: Option<i64>,
    #[schema(required = true, examples("John", "Maria"))]
    pub(crate) name: String,
}

#[rwf_macros::generate_full_model(i64, OrderItemController, order_items)]
#[derive(Clone, Model, Debug, Serialize, Deserialize, ToSchema, ToResponse)]
#[belongs_to(Order)]
#[belongs_to(Product)]
#[allow(dead_code)]
pub struct OrderItem {
    #[schema(minimum = 1, example = 128, format = "Int64")]
    pub(crate) id: Option<i64>,
    pub(crate) order_id: i64,
    pub(crate) product_id: i64,
    amount: f64,
}

#[rwf_macros::generate_full_model(i64, ProductController, produucts)]
#[derive(Clone, Model, Debug, Serialize, Deserialize, ToSchema, ToResponse)]
#[has_many(OrderItem)]
#[allow(dead_code)]
pub struct Product {
    pub(crate) id: Option<i64>,
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
#[rwf_macros::generate_full_model(i64, OrderController, orders)]
#[derive(Clone, Model, Debug, Serialize, Deserialize, ToSchema, ToResponse)]
#[belongs_to(User)]
#[has_many(OrderItem)]
#[allow(dead_code)]
#[schema(
    title = "An exaple Order",
    description = "A Order in the DB System. Referebces the user who made the order and is refereenced by all related order items"
)]
#[response(
    description = "Rerpresentation of a single Order azzoziated with the buying User and ordere3d Items"
)]
pub struct Order {
    #[schema(minimum = 1, example = 128, format = "Int64")]
    pub(crate) id: Option<i64>,
    #[schema(minimum = 1, example = 32, format = "Int64")]
    pub(crate) user_id: i64,
    pub(crate) name: String,
    #[schema(required = false, nullable = true)]
    pub(crate) optional: Option<String>,
}

impl OrderItem {
    pub fn expensive() -> Scope<Self> {
        Self::all().filter_gt("amount", 5.0)
    }
}
