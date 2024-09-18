use async_trait::async_trait;

pub mod auth;
pub mod error;
pub use auth::{AllowAll, Authentication, DenyAll};
pub use error::Error;

use super::http::{Method, Request, Response, ToResource};
use super::model::{get_connection, Model, Query, ToValue, Update};

use serde::{Deserialize, Serialize};

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

    fn auth(&self) -> Box<dyn Authentication> {
        Box::new(AllowAll {})
    }

    /// Figure out which method to call based on request method
    /// and path.
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        if !self.auth().authorize(request).await? {
            return Ok(Response::not_authorized());
        }

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
    type Model: Model + Serialize + Send + Sync + for<'a> Deserialize<'a>;

    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        if !self.auth().authorize(request).await? {
            return Ok(Response::not_authorized());
        }

        let method = request.method();
        if request.path().is_root() {
            match method {
                Method::Get => ModelController::list(self, request).await,
                Method::Post => ModelController::create(self, request).await,
                _ => Ok(Response::method_not_allowed()),
            }
        } else {
            let resource = request.path().resource::<Self::Resource>();
            if let Some(Ok(id)) = resource {
                match method {
                    Method::Get => ModelController::get(self, request, &id).await,
                    Method::Put => ModelController::update(self, request, &id).await,
                    Method::Delete => self.delete(request, &id).await,
                    Method::Patch => ModelController::patch(self, request, &id).await,
                    _ => Ok(Response::method_not_allowed()),
                }
            } else {
                Ok(Response::bad_request())
            }
        }
    }

    async fn list(&self, _request: &Request) -> Result<Response, Error> {
        let conn = get_connection().await?;

        let models = Self::Model::all().fetch_all(&conn).await?;
        let response = match Response::json(models) {
            Ok(response) => response,
            Err(err) => Response::internal_error(err),
        };

        Ok(response)
    }

    async fn get(&self, _request: &Request, id: &i64) -> Result<Response, Error> {
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

    async fn create(&self, request: &Request) -> Result<Response, Error> {
        let model = request.json::<Self::Model>()?;
        let conn = get_connection().await?;
        let model = model.create().fetch(&conn).await?;
        Ok(Response::json(model)?)
    }

    async fn update(&self, request: &Request, id: &i64) -> Result<Response, Error> {
        // The REST spec requires the entire model to be sent over for a PUT.
        let model = request.json::<Self::Model>()?;

        // The id field is immutable, but let's do a sanity check here just to
        // be sure the client sent the right model.
        if model.id() != Some(*id) {
            return Ok(Response::bad_request());
        }

        let conn = get_connection().await?;
        let model = model.save().fetch(&conn).await?;
        Ok(Response::json(model)?)
    }

    async fn patch(&self, request: &Request, id: &i64) -> Result<Response, Error> {
        let conn = get_connection().await?;
        let exists = Self::Model::find(*id).count(&conn).await?;

        if exists == 0 {
            return Ok(Response::not_found());
        }

        let req = match request.json_raw()?.as_object() {
            Some(req) => req.clone(),
            None => return Ok(Response::bad_request()),
        };

        let (mut columns, mut values) = (vec![], vec![]);

        for column in Self::Model::column_names() {
            if let Some(value) = req.get(&column) {
                let value = value.to_value();
                columns.push(column);
                values.push(value);
            }
        }

        let model = Query::Update(Update::<Self::Model>::from_columns(*id, &columns, &values))
            .fetch(&conn)
            .await?;

        Ok(Response::json(model)?)
    }
}
