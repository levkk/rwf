//! Signup controller.

use rwf::controller::middleware::prelude::*;
use rwf::prelude::*;

use crate::models::User;

mod form;
mod middleware;

use form::SignupForm;
use middleware::LoggedInCheck;

/// Handle user signup.
#[derive(rwf::macros::PageController)]
#[middleware(middleware)]
pub struct SignupController {
    middleware: MiddlewareSet,
}

impl Default for SignupController {
    fn default() -> SignupController {
        SignupController {
            middleware: MiddlewareSet::new(vec![LoggedInCheck::default().middleware()]),
        }
    }
}

#[rwf::async_trait]
impl PageController for SignupController {
    /// Respond to GET request.
    async fn get(&self, _request: &Request) -> Result<Response, Error> {
        let rendered = Template::load("templates/signup.html")?.render([("title", "Test")])?;

        Ok(Response::new().html(rendered))
    }

    /// Respond to POST request.
    async fn post(&self, request: &Request) -> Result<Response, Error> {
        let form = request.form::<SignupForm>()?;

        // <input required> wasn't respected.
        if form.name.is_empty() {
            return Ok(Response::bad_request());
        }

        let user = Pool::pool()
            .with_transaction(|mut transaction| async move {
                // Get or create user.
                let users = User::find_or_create_by(&[("name", form.name)])
                    .unique_by(&["name"])
                    .fetch(&mut transaction)
                    .await?;

                // Commit the transaction,
                // otherwise changes are automatically rolled back.
                transaction.commit().await?;
                Ok(users)
            })
            .await?;

        Ok(request.login(user.id.unwrap()).redirect("/chat"))
    }
}

// Log the user out.
#[derive(Default)]
pub struct LogoutController;

#[rwf::async_trait]
impl Controller for LogoutController {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        // Remove the user session from the cookie
        // and redirect to signup.
        Ok(request.logout().redirect("/signup"))
    }
}
