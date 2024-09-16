use async_trait::async_trait;
use std::str::FromStr;

pub mod error;
pub use error::Error;

use super::http::{Method, Request, Response};
use super::model::Model;

use std::fmt::Debug;

#[async_trait]
pub trait Controller: Sync {
    type Resource: FromStr + Send;

    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let method = request.method();
        if request.path().is_root() {
            match method {
                Method::Get => self.list(request).await,
                Method::Post => self.create(request).await,
                _ => Ok(Response::method_not_allowed()),
            }
        } else {
            let resource = request.path().resource::<Self::Resource>();
            if let Some(Ok(id)) = resource {
                match method {
                    Method::Get => self.get(request, &id).await,
                    Method::Put => self.update(request, &id).await,
                    Method::Delete => self.delete(request, &id).await,
                    _ => Ok(Response::method_not_allowed()),
                }
            } else {
                Ok(Response::bad_request())
            }
        }
    }

    async fn list(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::not_found())
    }

    async fn get(&self, request: &Request, id: &Self::Resource) -> Result<Response, Error> {
        Ok(Response::not_found())
    }

    async fn create(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::not_found())
    }

    async fn update(&self, request: &Request, id: &Self::Resource) -> Result<Response, Error> {
        Ok(Response::not_found())
    }

    async fn delete(&self, request: &Request, id: &Self::Resource) -> Result<Response, Error> {
        Ok(Response::not_found())
    }
}

#[async_trait]
pub trait ModelController: Controller {
    type Model: Model;
}
