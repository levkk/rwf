use rwf::model::Join;
use rwf::model::select::Op;
use rwf::prelude::*;

#[derive(Debug, Serialize, Deserialize, macros::Model, Clone, PartialEq, Eq)]
#[has_many(Order)]
pub struct User {
    id: Option<i64>,
    username: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize, macros::Model, Clone, PartialEq, Eq)]
#[belongs_to(User)]
pub struct Order {
    id: Option<i64>,
    user_id: i64,
    expensive: bool,
}
impl Order {
    pub fn latest_user_order_id() -> Scope<Self> {
        Self::all()
            .group_by(&["user_id"])
            .select_aggregated(&[("id", "MAX", Some("id"))])
    }
    pub fn latest_user_order() -> Scope<Self> {
        Self::all()
            .with(Order::latest_user_order_id(), "latest_id")
            .add_join(Join::new(Self::table_name(), "latet_id", "id", "id"))
    }
}

#[tokio::main]
async fn main() -> Result<(), rwf::model::Error> {
    Ok(())
}
