//! HTTP request routing.
//!
//! Rwf uses a global Regex to route requests. The Regex is construted at server
//! start from registered controllers.
//!
//! Using a regex is fast and can handle many complex use cases required for HTTP routing.
//!
//! ### Routing algorithm
//!
//! The routing algorithm is pretty straight forward. First, the path is matched against the global regex.
//! All matches are then ranked by their rank (which is configurable, and set to `-20` by default), and then
//! by the length of the path. Longer paths are assumed to be more specific and more correct.
//!
//! If multiple controllers match a path, the last one added to the router is returned. This is ensured by the stable
//! sorting property used by the router.
//!
//! ### Safety note
//!
//!
//! The route is a string, which accepts any valid regex. An issue can arise if the regex uses backtraces incorrectly,
//! so to make sure the Rwf server is immute to DoS attacks, use relatively simple regexes in your routing.
//!
//! Currently, Rwf makes no effort to protect against poorly constructed regexes by the user. This will change
//! in the future.
//!
use super::{Error, Handler, Path};
use crate::colors::MaybeColorize;

use regex::RegexSet;
use tracing::info;

/// The HTTP request router.
#[derive(Default)]
pub struct Router {
    regex: RegexSet,
    handlers: Vec<Handler>,
}

impl Router {
    /// Create new router from a list of handlers.
    pub fn new(handlers: Vec<Handler>) -> Result<Self, Error> {
        let paths = handlers
            .iter()
            .map(|h| h.path_with_regex().regex().as_str())
            .collect::<Vec<_>>();
        let regex = RegexSet::new(paths)?;

        Ok(Self { regex, handlers })
    }

    /// Find the best handler for the request path.
    ///
    /// See [`crate::http::router`] documentation for route matching algorithm description.
    pub fn find(&self, path: &Path) -> Option<&Handler> {
        let matches = self.regex.matches(path.base());
        let mut handlers = self
            .handlers
            .iter()
            .enumerate()
            .filter(|(i, _h)| matches.matched(*i))
            .map(|(_i, h)| h)
            .collect::<Vec<_>>();
        handlers.sort_by(|a, b| {
            let a_len = a.path().base().len();
            let b_len = b.path().base().len();
            let a_rank = a.rank();
            let b_rank = b.rank();

            if a_rank == b_rank {
                a_len.cmp(&b_len)
            } else {
                a_rank.cmp(&b_rank)
            }
        }); // Get the most specific path (longest match).
        handlers.last().copied()
    }

    /// Pretty print all registered routes.
    ///
    /// Used at server startup.
    pub fn log_routes(&self) {
        let mut handlers = self.handlers.iter().map(|s| s).collect::<Vec<_>>();
        handlers.sort_by_key(|s| s.path().path());
        for handler in handlers {
            // #[cfg(debug_assertions)]
            // let regex = format!(" ({})", handler.path_with_regex().regex().as_str());

            // #[cfg(not(debug_assertions))]
            // let regex = "";
            info!(
                ">> {} => {}",
                handler.path().path().purple(),
                handler.controller_name().green(),
                // regex,
            );
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::async_trait;
    use crate::controller::{Controller, Error as ControllerError};
    use crate::http::{Request, Response};

    struct OrdersControler {}
    struct UsersController {}

    #[async_trait]
    impl Controller for OrdersControler {
        async fn handle(&self, _request: &Request) -> Result<Response, ControllerError> {
            Ok(Response::default().text("OrdersControler"))
        }
    }

    #[async_trait]
    impl Controller for UsersController {
        async fn handle(&self, _request: &Request) -> Result<Response, ControllerError> {
            Ok(Response::default().text("UsersController"))
        }
    }

    #[tokio::test]
    async fn test_find() {
        let handler = Router::new(vec![
            OrdersControler {}.route("/api/orders"),
            UsersController {}.route("/api/users"),
        ])
        .expect("to compile");

        let handler = handler
            .find(&Path::parse("/api/orders").unwrap())
            .expect("to match");
        let result = handler.handle(&Request::default()).await.unwrap();
        assert_eq!(result.status().code(), 200);
    }
}
