use super::Error;
use crate::http::{Request, Response};
use async_trait::async_trait;

/// The result of middleware processing a request.
///
/// The middleware can either forward the request to the next middleware,
/// or block it and return its own response. Forwarded requests can be modified,
/// adding/removing headers or changing the body.
pub enum Outcome {
    Forward(Request),
    Block(Response),
}

#[async_trait]
pub trait Middleware: Send + Sync {
    async fn handle(&self, mut request: Request) -> Result<Outcome, Error>;
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

    async fn handle(&self, request: Request) -> Result<Outcome, Error> {
        self.middleware.handle(request).await
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

    pub async fn handle(&self, mut request: Request) -> Result<Outcome, Error> {
        for middleware in &self.handlers {
            match middleware.handle(request).await? {
                Outcome::Forward(req) => request = req,
                Outcome::Block(response) => return Ok(Outcome::Block(response)),
            }
        }

        Ok(Outcome::Forward(request))
    }
}
