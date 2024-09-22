use async_trait::async_trait;

pub mod auth;
pub mod error;
pub use auth::{AllowAll, AuthMechanism, Authentication, BasicAuth, DenyAll};
pub use error::Error;

use super::http::{Method, Request, Response, ToParameter};
use super::model::{get_connection, Model, Query, ToValue, Update};
use crate::config::get_config;

use serde::{Deserialize, Serialize};

/// The HTTP controller.
///
/// The most basic version of a controller handles all requests
/// which match the path it's assigned to.
///
/// Authentication is built-in and is configurable.
#[async_trait]
pub trait Controller: Sync + Send {
    /// Set the authentication mechanism for this controller.
    ///
    /// Default authentication method is to allow all requests, but can
    /// be adjusted through configuration.
    fn auth(&self) -> &Box<dyn Authentication> {
        // Allow all requests by default.
        let auth = &get_config().default_auth;
        auth.auth()
    }

    /// Internal function to handle the HTTP request. Do not implement this unless
    /// you're looking to do something really custom.
    async fn handle_internal(&self, request: &Request) -> Result<Response, Error> {
        let auth = self.auth();

        if !auth.authorize(request).await? {
            return auth.denied(request).await;
        }

        self.handle(request).await
    }

    /// Handle the request. Implement this function to define how your controller
    /// will respond to requests.
    async fn handle(&self, request: &Request) -> Result<Response, Error>;

    /// The name of this controller. Used for logging.
    fn controller_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// REST, aka CRUD, controller.
///
/// This controller will split incoming requests based on the REST specification and route
/// them to their respective methods.
///
/// Available methods are:
///
/// - list (GET /)
/// - create (POST /)
/// - get (GET /:id)
/// - update (PUT /:id)
/// - patch (PATCH /:id)
/// - delete (DELETE /:id)
///
/// By default, all methods will respond with 501 - Not Implemented. It's up to the user
/// to implement each method according to their needs.
///
/// The `:id` can be any value which implements the [`ToParameter`] trait.
/// Common data types are implemented, e.g. i64, String, etc.
///
/// # Example
///
/// ```
/// use rum::controller::{Controller, RestController, Error};
/// use rum::http::{Request, Response};
/// use rum::async_trait;
///
/// struct MyController {}
///
/// #[async_trait]
/// impl Controller for MyController {
///     // Delegate handling of this controller to the `RestController`.
///     async fn handle(&self, request: &Request) -> Result<Response, Error> {
///         RestController::handle(self, request).await
///     }  
/// }
///
/// #[async_trait]
/// impl RestController for MyController {
///     type Resource = i64;
///
///     async fn get(&self, request: &Request, id: &i64) -> Result<Response, Error> {
///         Ok(Response::html(format!("Hello, id #{}", id)))
///     }
/// }
/// ```
#[async_trait]
#[allow(unused_variables)] // Easier to read the code without _var_name.
pub trait RestController: Controller {
    type Resource: ToParameter;

    /// Figure out which method to call based on request method
    /// and path.
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let method = request.method();
        let parameter = request.parameter::<Self::Resource>(0);

        match parameter {
            Ok(Some(id)) => match method {
                Method::Get => self.get(request, &id).await,
                Method::Put => self.update(request, &id).await,
                Method::Delete => self.delete(request, &id).await,
                Method::Patch => self.patch(request, &id).await,
                _ => Ok(Response::method_not_allowed()),
            },
            Ok(None) => match method {
                Method::Get => self.list(request).await,
                Method::Post => self.create(request).await,
                _ => Ok(Response::method_not_allowed()),
            },
            _ => Ok(Response::bad_request()),
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

/// The model controller extends the [`RestController`] to
/// automatically performs CRUD actions on database models.
#[async_trait]
pub trait ModelController: Controller + RestController<Resource = i64> {
    type Model: Model + Serialize + Send + Sync + for<'a> Deserialize<'a>;

    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let method = request.method();
        let parameter = request.parameter::<Self::Resource>(0);

        match parameter {
            Ok(Some(id)) => match method {
                Method::Get => ModelController::get(self, request, &id).await,
                Method::Put => ModelController::update(self, request, &id).await,
                Method::Delete => self.delete(request, &id).await,
                Method::Patch => ModelController::patch(self, request, &id).await,
                _ => Ok(Response::method_not_allowed()),
            },

            Ok(None) => match method {
                Method::Get => ModelController::list(self, request).await,
                Method::Post => ModelController::create(self, request).await,
                _ => Ok(Response::method_not_allowed()),
            },

            _ => Ok(Response::bad_request()),
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

        // Only accept columns we know about, ignore the rest.
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
