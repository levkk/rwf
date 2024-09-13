use async_trait::async_trait;

pub mod error;
pub use error::Error;

use super::http::{Request, Response};
use super::model::Model;

#[async_trait]
pub trait Controller {
    async fn list(&self, request: &Request) -> Result<Response, Error>;
    async fn get(&self, request: &Request, id: &str) -> Result<Response, Error>;
    async fn create(&self, request: &Request) -> Result<Response, Error>;
    async fn update(&self, request: &Request, id: &str) -> Result<Response, Error>;
    async fn delete(&self, request: &Request, id: &str) -> Result<Response, Error>;
}

#[async_trait]
pub trait ModelController: Controller {
    type Model: Model;
}
