use async_trait::async_trait;
use std::str::FromStr;

pub mod error;
pub use error::Error;

use super::http::{Method, Request, Response, ToResource};
use super::model::Model;

use std::fmt::Debug;

use serde::Serialize;

#[async_trait]
pub trait Controller: Sync + Send {
    async fn handle(&self, request: &Request) -> Result<Response, Error>;

    fn controller_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

#[async_trait]
#[allow(unused_variables)] // Easier to read the code without _var_name.
pub trait RestController: Controller {
    type Resource: ToResource;

    /// Figure out which method to call based on request method
    /// and path.
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
                    Method::Patch => self.patch(request, &id).await,
                    _ => Ok(Response::method_not_allowed()),
                }
            } else {
                Ok(Response::bad_request())
            }
        }
    }

    async fn list(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::not_implemented())
    }

    async fn get(&self, request: &Request, id: &Self::Resource) -> Result<Response, Error> {
        Ok(Response::not_implemented())
    }

    async fn create(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::not_implemented())
    }

    async fn update(&self, request: &Request, id: &Self::Resource) -> Result<Response, Error> {
        Ok(Response::not_implemented())
    }

    async fn patch(&self, request: &Request, id: &Self::Resource) -> Result<Response, Error> {
        Ok(Response::not_implemented())
    }

    async fn delete(&self, request: &Request, id: &Self::Resource) -> Result<Response, Error> {
        Ok(Response::not_implemented())
    }
}

#[async_trait]
pub trait ModelController: Controller + RestController<Resource = i64> {
    type Model: Model + Serialize + Send + Sync;

    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let method = request.method();
        if request.path().is_root() {
            match method {
                Method::Get => ModelController::list(self, request).await,
                Method::Post => self.create(request).await,
                _ => Ok(Response::method_not_allowed()),
            }
        } else {
            let resource = request.path().resource::<Self::Resource>();
            if let Some(Ok(id)) = resource {
                match method {
                    Method::Get => ModelController::get(self, request, &id).await,
                    Method::Put => self.update(request, &id).await,
                    Method::Delete => self.delete(request, &id).await,
                    Method::Patch => self.patch(request, &id).await,
                    _ => Ok(Response::method_not_allowed()),
                }
            } else {
                Ok(Response::bad_request())
            }
        }
    }

    async fn list(&self, request: &Request) -> Result<Response, Error> {
        use crate::model::pool::get_connection;
        let conn = get_connection().await?;

        let models = Self::Model::all().fetch_all(&conn).await?;
        let response = match Response::json(models) {
            Ok(response) => response,
            Err(err) => Response::internal_error(err),
        };

        Ok(response)
    }

    async fn get(&self, request: &Request, id: &i64) -> Result<Response, Error> {
        use crate::model::pool::get_connection;

        let conn = get_connection().await?;

        match Self::Model::find_by(Self::Model::primary_key(), *id)
            .fetch(&conn)
            .await
        {
            Ok(model) => match Response::json(model) {
                Ok(response) => Ok(response),
                Err(err) => Ok(Response::internal_error(err)),
            },

            Err(_) => Ok(Response::not_found()),
        }
    }
}
