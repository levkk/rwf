use rum::http::Server;
use rum::prelude::*;
use serde::{Deserialize, Serialize};

mod secure;
use secure::SecureUserController;

/// The user model.
#[derive(Clone, rum::macros::Model, Serialize, Deserialize)]
struct User {
    // Ignore the "id" column in API requests,
    // the database assigns primary keys.
    #[serde(skip_deserializing)]
    id: Option<i64>,

    // Required email field.
    email: String,

    // Optional "created_at" column. It's not null in the database,
    // but optional at the API, with serde (the deserializer) setting it
    // to "now" automatically.
    #[serde(with = "time::serde::iso8601", default = "OffsetDateTime::now_utc")]
    created_at: OffsetDateTime,
}

#[derive(Default, rum::macros::ModelController)]
struct UserController;

/// The model controller which automatically implements
/// all CRUD (create, read, update, destroy) actions
/// for this model.
#[async_trait]
impl ModelController for UserController {
    type Model = User;
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Configure logging.
    Logger::init();

    Server::new(vec![
        UserController::default().crud("/api/users"),
        SecureUserController::new().crud("/api/users/secure"),
    ])
    .launch("0.0.0.0:8000")
    .await?;

    Ok(())
}
