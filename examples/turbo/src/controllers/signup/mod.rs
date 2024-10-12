//! Signup controller.

use rwf::controller::middleware::prelude::*;
use rwf::prelude::*;

use crate::models::User;

use std::collections::HashMap;

mod form;
mod middleware;

use form::SignupForm;
use middleware::LoggedInCheck;

/// Handle user signup.
pub struct SignupController {
    middleware: MiddlewareSet,
}

impl SignupController {
    pub fn new() -> SignupController {
        SignupController {
            middleware: MiddlewareSet::new(vec![LoggedInCheck::default().middleware()]),
        }
    }
}

#[rwf::async_trait]
impl Controller for SignupController {
    fn middleware(&self) -> &MiddlewareSet {
        &self.middleware
    }

    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        PageController::handle(self, request).await
    }
}

#[rwf::async_trait]
impl PageController for SignupController {
    async fn get(&self, _request: &Request) -> Result<Response, Error> {
        let rendered =
            Template::load("templates/signup.html")?.render(HashMap::from([("title", "Test")]))?;

        Ok(Response::new().html(rendered))
    }

    async fn post(&self, request: &Request) -> Result<Response, Error> {
        let form = request.form::<SignupForm>()?;

        // Browsers set an empty field
        if form.name.is_empty() {
            return Ok(Response::bad_request());
        }

        let user = Pool::pool()
            .with_transaction(|mut transaction| async move {
                User::find_or_create_by(&[("name", form.name)])
                    .unique_by(&["name"])
                    .fetch(&mut transaction)
                    .await
            })
            .await?;

        Ok(request.login(user.id.unwrap()).redirect("/chat"))
    }
}
