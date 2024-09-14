use async_trait::async_trait;

pub mod error;
pub use error::Error;

use super::http::{Request, Response};
use super::model::Model;

#[async_trait]
pub trait Controller: Sync {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        if request.path().is_root() {}

        todo!()
    }

    async fn list(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::not_found())
    }

    async fn get(&self, request: &Request, id: &str) -> Result<Response, Error> {
        Ok(Response::not_found())
    }

    async fn create(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::not_found())
    }

    async fn update(&self, request: &Request, id: &str) -> Result<Response, Error> {
        Ok(Response::not_found())
    }

    async fn delete(&self, request: &Request, id: &str) -> Result<Response, Error> {
        Ok(Response::not_found())
    }
}

#[async_trait]
pub trait ModelController: Controller {
    type Model: Model;
}
