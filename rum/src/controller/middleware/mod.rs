use super::Error;
use crate::http::{Request, Response};
use async_trait::async_trait;

pub mod rate_limiter;
pub use rate_limiter::RateLimiter;

pub mod prelude;
pub mod secure_id;

/// The result of middleware processing a request.
///
/// The middleware can either forward the request to the next middleware,
/// or block it and return its own response. Forwarded requests can be modified,
/// adding/removing headers or changing the body.
pub enum Outcome {
    Forward(Request),
    Stop(Response),
}

#[async_trait]
#[allow(unused_variables)]
pub trait Middleware: Send + Sync {
    async fn handle_request(&self, request: Request) -> Result<Outcome, Error>;
    async fn handle_response(
        &self,
        request: &Request,
        response: Response,
    ) -> Result<Response, Error> {
        Ok(response)
    }
}

pub struct MiddlewareHandler {
    middleware: Box<dyn Middleware>,
}

impl MiddlewareHandler {
    pub fn new(middleware: impl Middleware + 'static) -> Self {
        Self {
            middleware: Box::new(middleware),
        }
    }

    async fn handle_request(&self, request: Request) -> Result<Outcome, Error> {
        self.middleware.handle_request(request).await
    }

    async fn handle_response(
        &self,
        request: &Request,
        response: Response,
    ) -> Result<Response, Error> {
        self.middleware.handle_response(request, response).await
    }
}

#[derive(Default)]
pub struct MiddlewareSet {
    handlers: Vec<MiddlewareHandler>,
}

impl MiddlewareSet {
    pub fn new(handlers: Vec<MiddlewareHandler>) -> Self {
        Self { handlers }
    }

    pub async fn handle_request(&self, mut request: Request) -> Result<Outcome, Error> {
        for middleware in &self.handlers {
            match middleware.handle_request(request).await? {
                Outcome::Forward(req) => request = req,
                Outcome::Stop(response) => return Ok(Outcome::Stop(response)),
            }
        }

        Ok(Outcome::Forward(request))
    }

    pub async fn handle_response(
        &self,
        request: &Request,
        mut response: Response,
    ) -> Result<Response, Error> {
        for middleware in self.handlers.iter().rev() {
            response = middleware.handle_response(request, response).await?;
        }

        Ok(response)
    }
}
