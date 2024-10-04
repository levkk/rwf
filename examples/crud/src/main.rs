use rum::http::Server;
use rum::logging::setup_logging;
use rum::prelude::*;
use serde::{Deserialize, Serialize};

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

#[derive(Default)]
struct UserController;

/// The model controller which automatically implements
/// all CRUD (create, read, update, destroy) actions
/// for this model.
#[async_trait]
impl ModelController for UserController {
    type Model = User;
}

/// CRUD is based on REST, for which we also have an automatically
/// implemented controller.
#[async_trait]
impl RestController for UserController {
    // REST actions will be addressed using the primary key
    // of the model (in our case, a 64-bit integer).
    type Resource = i64;

    // Delegate handling of REST actions to the ModelController.
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        ModelController::handle(self, request).await
    }
}

/// All routes in Rum have to implement the Controller trait.
/// We delegate this implementation to the ModelController.
#[async_trait]
impl Controller for UserController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        ModelController::handle(self, request).await
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Configure logging.
    setup_logging();

    Server::new(vec![UserController::default().route("/api/users")])
        .launch("0.0.0.0:8000")
        .await?;

    Ok(())
}
