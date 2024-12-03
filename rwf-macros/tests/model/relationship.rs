use rwf_macros::*;

#[derive(Model, Clone)]
#[has_many(Task)]
pub struct User {
    id: Option<i64>,
    email: String,
}

#[derive(Model, Clone)]
#[belongs_to(User)]
pub struct Task {
    id: Option<i64>,
    user_id: i64,
}

fn main() {}
