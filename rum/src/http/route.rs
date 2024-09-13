use super::{Error, Request, Response};

use std::future::Future;

use tokio::task::JoinHandle;

pub trait Route: Clone + Send + 'static {
    fn handle(&self, request: Request) -> impl Future<Output = Result<Response, Error>> + Send;

    fn path(&self) -> &'static str;

    fn execute_internal(&self, request: Request) -> JoinHandle<Result<Response, Error>> {
        let route = self.clone();
        tokio::spawn(async move { route.handle(request).await })
    }
}
