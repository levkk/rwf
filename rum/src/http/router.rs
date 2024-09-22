use super::{Error, Handler, Path};

use regex::RegexSet;

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

    pub fn find(&self, path: &Path) -> Result<Option<&Handler>, Error> {
        let matches = self.regex.matches(path.base());
        let mut handlers = self
            .handlers
            .iter()
            .enumerate()
            .filter(|(i, _h)| matches.matched(*i))
            .map(|(_i, h)| h)
            .collect::<Vec<_>>();
        handlers.sort_by_key(|h| h.path().base().len()); // Get the most specific path (longest match).
        Ok(handlers.last().copied())
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
            Ok(Response::text("OrdersControler"))
        }
    }

    #[async_trait]
    impl Controller for UsersController {
        async fn handle(&self, _request: &Request) -> Result<Response, ControllerError> {
            Ok(Response::text("UsersController"))
        }
    }

    #[tokio::test]
    async fn test_find() {
        let handler = Router::new(vec![
            Handler::new("/api/orders", OrdersControler {}),
            Handler::new("/api/users", UsersController {}),
        ])
        .expect("to compile");

        let handler = handler
            .find(&Path::parse("/api/orders").unwrap())
            .expect("to match")
            .unwrap();
        let result = handler.handle(&Request::default()).await.unwrap();
        assert_eq!(result.as_str().unwrap(), "OrdersControler");
    }
}
