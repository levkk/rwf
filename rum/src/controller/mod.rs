use async_trait::async_trait;

pub mod auth;
pub mod error;
pub mod middleware;
pub mod static_files;
pub mod util;

pub use auth::{AllowAll, AuthHandler, Authentication, BasicAuth, DenyAll, Session, SessionId};
pub use error::Error;
pub use middleware::{Middleware, MiddlewareHandler, MiddlewareSet, Outcome, RateLimiter};
pub use static_files::StaticFiles;

use super::http::{
    websocket::{self, DataFrame},
    Handler, Method, Protocol, Request, Response, Stream, ToParameter,
};
use super::model::{get_connection, Insert, Model, Query, ToValue, Update, Value};
use crate::comms::get_comms;
use crate::config::get_config;

use tokio::select;
use tokio::time::{interval, timeout};
use tracing::debug;

use serde::{Deserialize, Serialize};

/// The HTTP controller.
///
/// The most basic version of a controller handles all requests
/// which match the path it's assigned to.
///
/// Authentication is built-in and is configurable.
#[async_trait]
#[allow(unused_variables)]
pub trait Controller: Sync + Send {
    /// Set the authentication mechanism for this controller.
    ///
    /// Default authentication method is to allow all requests, but can
    /// be adjusted through configuration.
    fn auth(&self) -> &AuthHandler {
        // Allow all requests by default.
        &get_config().default_auth
    }

    fn middleware(&self) -> &MiddlewareSet {
        &get_config().default_middleware
    }

    fn route(self, path: &str) -> Handler
    where
        Self: Sized + 'static,
    {
        Handler::route(path, self)
    }

    fn wildcard(self, path: &str) -> Handler
    where
        Self: Sized + 'static,
    {
        todo!()
    }

    fn protocol(&self) -> Protocol {
        Protocol::Http1
    }

    /// Handle the TCP connection directly.
    async fn handle_stream(&self, request: &Request, stream: Stream<'_>) -> Result<bool, Error> {
        Ok(request.keep_alive())
    }

    /// Internal function to handle the HTTP request. Do not implement this unless
    /// you're looking to do something really custom.
    async fn handle_internal(&self, request: Request) -> Result<Response, Error> {
        let auth = self.auth();

        if !auth.auth().authorize(&request).await? {
            return auth.auth().denied(&request).await;
        }

        let no_session = request.session().is_none();

        let outcome = self.middleware().handle_request(request).await?;
        let response = match outcome {
            Outcome::Forward(request) => match self.handle(&request).await {
                Ok(response) => {
                    self.middleware()
                        .handle_response(&request, response.from_request(&request)?)
                        .await?
                }
                Err(err) => return Err(err),
            },
            Outcome::Stop(response) => response,
        };

        Ok(response)
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
///     async fn handle(&self, request: &Request) -> Result<Response, Error> {
///         // Delegate handling of this controller to the `RestController`.
///         RestController::handle(self, request).await
///     }  
/// }
///
/// #[async_trait]
/// impl RestController for MyController {
///     type Resource = i64;
///
///     async fn get(&self, request: &Request, id: &i64) -> Result<Response, Error> {
///         Ok(Response::default().html(format!("Hello, id #{}", id)))
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
        let parameter = request.parameter::<Self::Resource>("id");

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

    fn rest(self, path: &str) -> Handler
    where
        Self: Sized + 'static,
    {
        Handler::rest(path, self)
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
pub trait ModelController: Controller {
    type Model: Model + Serialize + Send + Sync + for<'a> Deserialize<'a>;

    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let method = request.method();
        let parameter = request.parameter::<i64>("id");

        match parameter {
            Ok(Some(id)) => match method {
                Method::Get => ModelController::get(self, request, &id).await,
                Method::Put => ModelController::update(self, request, &id).await,
                Method::Delete => return Ok(Response::not_found()),
                Method::Patch => ModelController::patch(self, request, &id).await,
                _ => Ok(Response::method_not_allowed()),
            },

            Ok(None) => match method {
                Method::Get => ModelController::list(self, request).await,
                Method::Post => ModelController::create(self, request).await,
                _ => Ok(Response::method_not_allowed()),
            },

            Err(_) => Ok(Response::bad_request()),
        }
    }

    fn crud(self, path: &str) -> Handler
    where
        Self: Sized + 'static,
    {
        Handler::rest(path, self)
    }

    async fn list(&self, request: &Request) -> Result<Response, Error> {
        let mut conn = get_connection().await?;
        let page_size = request.query().get::<i64>("page_size").unwrap_or(25);
        let page = request.query().get::<i64>("page").unwrap_or(1);
        let offset = (std::cmp::min(1, page) - 1) * page_size;

        let models = Self::Model::all()
            .limit(page_size)
            .offset(offset)
            .fetch_all(&mut conn)
            .await?;
        let response = match Response::new().json(models) {
            Ok(response) => response,
            Err(err) => Response::internal_error(err),
        };

        Ok(response)
    }

    async fn get(&self, _request: &Request, id: &i64) -> Result<Response, Error> {
        let mut conn = get_connection().await?;

        match Self::Model::find_by(Self::Model::primary_key(), *id)
            .fetch(&mut conn)
            .await
        {
            Ok(model) => match Response::new().json(model) {
                Ok(response) => Ok(response),
                Err(err) => Ok(Response::internal_error(err)),
            },

            Err(_) => Ok(Response::not_found()),
        }
    }

    async fn create(&self, request: &Request) -> Result<Response, Error> {
        let model = match request.json::<Self::Model>() {
            Ok(model) => model,
            Err(_err) => {
                return Ok(Response::bad_request());
            }
        };

        let mut conn = get_connection().await?;

        let model = Query::Insert(Insert::<Self::Model>::from_columns(
            &Self::Model::column_names(),
            &model.values(),
        ))
        .fetch(&mut conn)
        .await?;

        Ok(Response::new().json(model)?)
    }

    async fn update(&self, request: &Request, id: &i64) -> Result<Response, Error> {
        // The REST spec requires the entire model to be sent over for a PUT.
        let model = request.json::<Self::Model>()?;

        // The id field is immutable, but let's do a sanity check here just to
        // be sure the client sent the right model.
        if model.id() != Value::Integer(*id) {
            return Ok(Response::bad_request());
        }

        let mut conn = get_connection().await?;
        let model = model.save().fetch(&mut conn).await?;
        Ok(Response::new().json(model)?)
    }

    async fn patch(&self, request: &Request, id: &i64) -> Result<Response, Error> {
        let mut conn = get_connection().await?;
        let exists = Self::Model::find(*id).count(&mut conn).await?;

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
            if let Some(value) = req.get(*column) {
                let value = value.to_value();
                columns.push(*column);
                values.push(value);
            }
        }

        let model = Query::Update(Update::<Self::Model>::from_columns(*id, &columns, &values))
            .fetch(&mut conn)
            .await?;

        Ok(Response::new().json(model)?)
    }
}

#[async_trait]
pub trait WebsocketController: Controller {
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        use base64::{engine::general_purpose, Engine as _};
        use sha1::{Digest, Sha1};

        if !request.upgrade_websocket() {
            return Ok(Response::bad_request());
        }

        let headers = match websocket::Headers::from_http_request(request) {
            Ok(headers) => headers,
            Err(_) => return Ok(Response::bad_request()),
        };

        let accept = headers.key.clone() + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
        let digest = Sha1::digest(accept.as_bytes());
        let base64 = general_purpose::STANDARD.encode(digest);

        Ok(Response::switching_protocols("websocket").header("sec-websocket-accept", base64))
    }

    async fn client_message(
        &self,
        session_id: &SessionId,
        message: websocket::Message,
    ) -> Result<(), Error>;

    async fn handle_stream(
        &self,
        request: &Request,
        mut stream: Stream<'_>,
    ) -> Result<bool, Error> {
        use tokio::sync::broadcast::error::RecvError;

        let session_id = if let Some(session) = request.session() {
            session.session_id.clone()
        } else {
            return Err(Error::SessionMissingError);
        };

        debug!("new websocket connection from session \"{:?}\"", session_id);

        let comms = get_comms();
        let config = get_config();
        let mut stream = stream.stream();
        let mut receiver = comms.websocket_receiver(&session_id);
        let mut check = interval(config.websocket.ping_interval.unsigned_abs());
        let mut lost_pings = 0;

        loop {
            select! {
                _ = check.tick() => {
                    debug!("checking websocket session \"{:?}\"", session_id);

                    let closed = match timeout(
                        config.websocket.ping_timeout.unsigned_abs(),
                        DataFrame::new_ping().flush(&mut stream)
                    ).await {
                        Ok(Ok(_)) => false,
                        _ => true,
                    };

                    lost_pings += 1;

                    if closed || lost_pings > config.websocket.ping_disconnect_count {
                        break;
                    }
                }

                message = receiver.recv() => {
                    match message {
                        Ok(message) => {
                            message.send(&mut stream).await?;
                        }

                        Err(RecvError::Closed) => break,

                        // Lagging behind. This is best effort
                        // message delivery, so we are ok dropping
                        // messages if the client can't receive them
                        // fast enough.
                        Err(RecvError::Lagged(_)) => continue,
                    }
                }

                frame = DataFrame::read(&mut stream) => {
                    let frame = frame?;

                    if frame.is_pong() {
                        debug!("websocket session \"{:?}\" is alive", session_id);
                        lost_pings -= 1;

                        // Protect against weird clients.
                        if lost_pings < 0 {
                            lost_pings = 0;
                        }

                        continue;
                    } else if frame.is_ping() {
                        DataFrame::new_pong(frame).flush(&mut stream).await?;
                        continue;
                    }

                    self.client_message(&session_id, frame.message()).await?;
                }

            }
        }

        Ok(false)
    }
}
