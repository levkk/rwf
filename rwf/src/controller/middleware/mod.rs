//! HTTP middleware.
//!
//! Middleware runs before a request reaches a controller and
//! after the controller has returned a response. It can transform requests, by adding a header for example,
//! or reject them altogether and return a different response. It can also transform responses, and perform actions
//! before and after the potentially modified response is passed down the middleware chain.
//!
//! Implementing your own middleware requires implementing the [`Middleware`] trait on a struct. Rwf comes with several predefined
//! middleware you can use for inspiration, e.g. [`RateLimiter`].
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

pub mod csrf;
pub mod request_tracker;

/// The result of middleware processing a request.
pub enum Outcome {
    /// Forward the request to the next middleware in the chain, or if none are left,
    /// to the controller.
    Forward(Request),
    /// Intercept the request, and return the response instead.
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

/// Wrapper around a struct implementing the [`Middleware`] trait.
///
/// The [`Middleware::middleware()`] method returns this wrapper, so you
/// don't need to construct this manually.
#[derive(Clone)]
pub struct MiddlewareHandler {
    middleware: Arc<Box<dyn Middleware>>,
}

impl MiddlewareHandler {
    /// Create new middleware wrapper.
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

/// A middleware collection. The middleware in this set is
/// executed in the specified order at creation.
#[derive(Default, Clone)]
pub struct MiddlewareSet {
    handlers: Vec<MiddlewareHandler>,
}

impl MiddlewareSet {
    /// Create new middleware set. This will include middleware that runs
    /// on every controller, if any is configured.
    pub fn new(handlers: Vec<MiddlewareHandler>) -> Self {
        let mut default_handlers = get_config().general.default_middleware.handlers();
        default_handlers.extend(handlers);

        Self {
            handlers: default_handlers,
        }
    }

    /// Create a middleware set without the default middleware that runs on every controller.
    /// Your controller will _only_ run your middleware, and included features
    /// like request tracking will be disabled on your controller.
    pub fn without_default(handlers: Vec<MiddlewareHandler>) -> Self {
        Self { handlers }
    }

    /// Handle an incoming request, by sending it through the middleware chain.
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

    /// Handle the response received from a controller, by sending it back
    /// through the middleware chain in reverse order.
    ///
    /// If a request was intercepted by a middleware in the chain, only
    /// the middleware that already ran will be executed.
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

    /// Returns a clone of the middleware wrappers. They cannot be executed manually.
    pub fn handlers(&self) -> Vec<MiddlewareHandler> {
        self.handlers.clone()
    }
}
