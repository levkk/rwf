use rwf::controller::middleware::prelude::*;

// If a user is already logged in, redirect them to the chat page.
#[derive(Default)]
pub struct LoggedInCheck;

#[rwf::async_trait]
impl Middleware for LoggedInCheck {
    async fn handle_request(&self, request: Request) -> Result<Outcome, Error> {
        if request.session().authenticated() {
            return Ok(Outcome::Stop(request, Response::new().redirect("/chat")));
        }

        Ok(Outcome::Forward(request))
    }
}
