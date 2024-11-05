use super::Error;
use crate::{
    colors::MaybeColorize,
    config::get_config,
    http::{Request, Response},
};
use async_trait::async_trait;
use std::ops::Deref;
use std::sync::Arc;
use tracing::debug;

pub mod rate_limiter;
pub use rate_limiter::RateLimiter;

pub mod prelude;

pub mod secure_id;
pub use secure_id::SecureId;

pub mod request_tracker;

/// The result of middleware processing a request.
///
/// The middleware can either forward the request to the next middleware,
/// or block it and return its own response. Forwarded requests can be modified,
/// adding/removing headers or changing the body.
pub enum Outcome {
    Forward(Request),
    Stop(Request, Response),
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

    fn middleware(self) -> MiddlewareHandler
    where
        Self: Sized + 'static,
    {
        MiddlewareHandler::new(self)
    }

    fn middleware_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[derive(Clone)]
pub struct MiddlewareHandler {
    middleware: Arc<Box<dyn Middleware>>,
}

impl MiddlewareHandler {
    pub fn new(middleware: impl Middleware + 'static) -> Self {
        Self {
            middleware: Arc::new(Box::new(middleware)),
        }
    }

    async fn handle_request(&self, request: Request) -> Result<Outcome, Error> {
        debug!(
            "{} {} => {}",
            "middleware".purple(),
            request.path().base().purple(),
            self.middleware.deref().middleware_name().green()
        );
        self.middleware.deref().handle_request(request).await
    }

    async fn handle_response(
        &self,
        request: &Request,
        response: Response,
    ) -> Result<Response, Error> {
        debug!(
            "{} {} <= {}",
            "middleware".purple(),
            request.path().base().purple(),
            self.middleware.deref().middleware_name().green()
        );
        self.middleware
            .deref()
            .handle_response(request, response)
            .await
    }
}

#[derive(Default, Clone)]
pub struct MiddlewareSet {
    handlers: Vec<MiddlewareHandler>,
}

impl MiddlewareSet {
    /// Create new middleware set, including middleware that runs by default
    /// on every controller.
    pub fn new(handlers: Vec<MiddlewareHandler>) -> Self {
        let mut default_handlers = get_config().general.default_middleware.handlers();
        default_handlers.extend(handlers);

        Self {
            handlers: default_handlers,
        }
    }

    /// Create a middleware set without the default middleware that runs on every controller.
    /// Your controller will _only_ run your middleware, and included features like analytics won't work on your controller.
    pub fn without_default(handlers: Vec<MiddlewareHandler>) -> Self {
        Self { handlers }
    }

    pub async fn handle_request(&self, mut request: Request) -> Result<(Outcome, usize), Error> {
        for (idx, middleware) in self.handlers.iter().enumerate() {
            match middleware.handle_request(request).await? {
                Outcome::Forward(req) => request = req,
                Outcome::Stop(request, response) => {
                    return Ok((Outcome::Stop(request, response), idx))
                }
            }
        }

        Ok((Outcome::Forward(request), self.handlers.len()))
    }

    pub async fn handle_response(
        &self,
        request: &Request,
        mut response: Response,
        executed: usize,
    ) -> Result<Response, Error> {
        // Skip middleware that didn't run because the request was stopped.
        let skip = self.handlers.len() - executed;
        for middleware in self.handlers.iter().rev().skip(skip) {
            response = middleware.handle_response(request, response).await?;
        }

        Ok(response)
    }

    pub fn handlers(&self) -> Vec<MiddlewareHandler> {
        self.handlers.clone()
    }
}
