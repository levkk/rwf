use rum::controller::middleware::{prelude::*, SecureId};
use rum::prelude::*;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Clone, Serialize, Deserialize, rum::macros::Model)]
pub struct User {
    #[serde(with = "rum::controller::ser::secure_id", default, skip_deserializing)]
    id: Option<i64>,

    email: String,

    #[serde(with = "time::serde::iso8601", default = "OffsetDateTime::now_utc")]
    created_at: OffsetDateTime,
}

pub struct SecureUserController {
    middleware: MiddlewareSet,
}

impl SecureUserController {
    pub fn new() -> SecureUserController {
        SecureUserController {
            middleware: MiddlewareSet::new(vec![SecureId::default().middleware()]),
        }
    }
}

#[rum::async_trait]
impl Controller for SecureUserController {
    fn middleware(&self) -> &MiddlewareSet {
        &self.middleware
    }

    /// Make the ModelController handle the request.
    /// This is required because Rust traits call the base trait method
    /// if it has a default implementation.
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        ModelController::handle(self, request).await
    }
}

impl ModelController for SecureUserController {
    type Model = User;
}
