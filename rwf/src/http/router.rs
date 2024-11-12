//! HTTP request routing.
//!
use super::{Error, Handler, Path};
use crate::colors::MaybeColorize;

use regex::RegexSet;
use tracing::info;

#[derive(Default)]
pub struct Router {
    regex: RegexSet,
    handlers: Vec<Handler>,
}

impl Router {
    pub fn new(handlers: Vec<Handler>) -> Result<Self, Error> {
        let paths = handlers
            .iter()
            .map(|h| h.path_with_regex().regex().as_str())
            .collect::<Vec<_>>();
        let regex = RegexSet::new(paths)?;

        Ok(Self { regex, handlers })
    }

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

    pub fn log_routes(&self) {
        let mut handlers = self.handlers.iter().map(|s| s).collect::<Vec<_>>();
        handlers.sort_by_key(|s| s.path().path());
        for handler in handlers {
            info!(
                ">> {} => {}",
                handler.path().path().purple(),
                handler.controller_name().green()
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
