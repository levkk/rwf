use rum::controller::{Middleware, MiddlewareSet, Outcome};
use rum::logging::setup_logging;
use rum::prelude::*;
use rum::Server;

#[derive(Default)]
struct BlockBadHeader;

#[rum::async_trait]
impl Middleware for BlockBadHeader {
    async fn handle_request(&self, request: Request) -> Result<Outcome, Error> {
        if let Some(value) = request.headers().get("x-user-id") {
            if let Ok(_id) = value.parse::<i64>() {
                return Ok(Outcome::Forward(request));
            }
        }

        Ok(Outcome::Stop(Response::bad_request()))
    }
}

struct IndexController {
    middleware: MiddlewareSet,
}

#[rum::async_trait]
impl Controller for IndexController {
    fn middleware(&self) -> &MiddlewareSet {
        &self.middleware
    }

    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Response::new().text("You are allowed in!"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    setup_logging();

    Server::new(vec![IndexController {
        middleware: MiddlewareSet::new(vec![BlockBadHeader::default().middleware()]),
    }
    .route("/")])
    .launch("0.0.0.0:8000")
    .await?;

    Ok(())
}
