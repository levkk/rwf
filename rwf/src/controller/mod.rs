//! HTTP controllers, the **C** in MVC.
//!
//! Controllers are any struct that implements the [`Controller`] trait. Rwf includes many prebuilt controllers
//! that perform useful tasks, like handling WebSocket connections or REST.
//!
//! A basic controller will just implement the [`Controller::handle`] method. The [`Controller`] trait is async, so we're using the [`async_trait::async_trait`] macro.
//!
//! #### Example
//!
//! ```rust
//! // Include all necessary types.
//! use rwf::prelude::*;
//!
//! // Your controller. The Default trait helps with easy instantiation.
//! #[derive(Default)]
//! struct Index;
//!
//! // Controllers are async and use the `async_trait` crate.
//! #[async_trait]
//! impl Controller for Index {
//!     async fn handle(&self, request: &Request) -> Result<Response, Error> {
//!         Ok(Response::new().html("<h1>Hello from Rwf!</h1>"))
//!     }
//! }
//! ```
//!
use async_trait::async_trait;

pub mod auth;
pub mod engine;
pub mod error;
pub mod middleware;
pub mod ser;
pub mod static_files;
pub mod turbo_stream;

#[cfg(feature = "wsgi")]
pub mod wsgi;
#[cfg(feature = "wsgi")]
pub use wsgi::WsgiController;

#[cfg(feature = "rack")]
pub mod rack;
#[cfg(feature = "rack")]
pub use rack::RackController;

pub use auth::{AllowAll, AuthHandler, Authentication, BasicAuth, DenyAll, Session, SessionId};
pub use engine::Engine;
pub use error::Error;
pub use middleware::{Middleware, MiddlewareHandler, MiddlewareSet, Outcome, RateLimiter};
pub use static_files::{CacheControl, StaticFiles};
pub use turbo_stream::TurboStream;

use super::http::{
    websocket::{self, DataFrame},
    Handler, Method, Request, Response, Stream, ToParameter,
};
use super::model::{get_connection, Insert, Model, Query, ToValue, Update, Value};
use crate::colors::MaybeColorize;
use crate::comms::Comms;
use crate::config::get_config;

use tokio::select;
use tokio::time::{interval, timeout};
use tracing::{debug, error, info};

use serde::{Deserialize, Serialize};

/// The controller, the **C** in MVC.
///
/// A controller handles an HTTP request routed to it by the server and returns
/// a response. Controllers in Rwf are asynchronous and use the `async_trait` crate. For this reason,
/// the trait signature looks a bit complicated, but underneath, all asynchronous functions are actually pretty simple.
///
/// ### Handling requests
///
/// The only function that requires implementation is the [`Controller::handle`] method. It receives a [`Request`] and
/// must return either a [`Response`] or an [`Error`].
///
/// ```rust
/// // Import required types and traits.
/// use rwf::prelude::*;
///
/// // A controller is a plain struct
/// // which implements the `Controller` trait.
/// struct Index;
///
/// // We use `async_trait` crate to handle async Rust traits.
/// #[async_trait]
/// impl Controller for Index {
///     // This method responds to all requests routed
///     // to this controller.
///     async fn handle(&self, request: &Request) -> Result<Response, Error> {
///         Ok(Response::new().html("<h1>Hello from Rwf!</h1>"))
///     }
/// }
/// ```
#[async_trait]
#[allow(unused_variables)]
pub trait Controller: Sync + Send {
    /// Set the authentication mechanism for this controller.
    /// Default authentication method is to allow all requests, but can
    /// be adjusted through configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rwf::prelude::*;
    /// use rwf::controller::DenyAll;
    ///
    /// // The auth handler should be defined on the controller struct.
    /// struct Index {
    ///     auth: AuthHandler,
    /// }
    ///
    /// // Auth handlers need to be instantiated.
    /// // Some like `BasicAuth` require parameters like username and password.
    /// impl Default for Index {
    ///     fn default() -> Self {
    ///         Self {
    ///             auth: AuthHandler::new(DenyAll {}),
    ///         }
    ///     }
    /// }
    ///
    /// #[async_trait]
    /// impl Controller for Index {
    ///     // Return the auth handler reference.
    ///     fn auth(&self) -> &AuthHandler {
    ///         &self.auth
    ///     }
    ///
    ///     async fn handle(&self, request: &Request) -> Result<Response, Error> {
    ///         todo!() // Handle request.
    ///     }
    /// }
    /// ```
    fn auth(&self) -> &AuthHandler {
        // Allow all requests by default.
        &get_config().general.default_auth
    }

    /// Configure middleware on this controller.
    /// Global middleware can be set in the configuration. By default,
    /// controllers have no middleware.
    fn middleware(&self) -> &MiddlewareSet {
        &get_config().general.default_middleware
    }

    /// Don't use [CSRF](https://owasp.org/www-community/attacks/csrf) protection on this controller. You generally don't want to disable this unless you
    /// have another mechanism to make sure your users are not being duped into making requests to your app
    /// from somewhere else.
    fn skip_csrf(&self) -> bool {
        false
    }

    /// Create a basic route handler for this controller.
    ///
    /// This method can be used to register a controller with the HTTP server.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// Index::new().route("/")
    /// ```
    ///
    /// is equivalent to using the [`rwf_macros::route`] macro:
    ///
    /// ```rust,ignore
    /// route!("/" => Index)
    /// ```
    fn route(self, path: &str) -> Handler
    where
        Self: Sized + 'static,
    {
        Handler::route(path, self)
    }

    /// Create a wildcard route handler for this controller.
    ///
    /// A wildcard handler will serve all requests that match this path and
    /// all paths that have this path as its parent. For example, if the path is set to
    /// `/users`, all paths that start with `/users`, like `/users/account`, `/users/5`, etc.,
    /// will be served by this controller.
    ///
    /// This is useful for creating catch-all controllers, and the handler
    /// will have the lowest rank in the [`crate::http::Router`].
    fn wildcard(self, path: &str) -> Handler
    where
        Self: Sized + 'static,
    {
        Handler::wildcard(path, self)
    }

    /// Internal function that handlers the TCP connection directly after a response
    /// has been sent to the client by the controller. This is typically used for WebSocket connections,
    /// but can also be used to stream data like video.
    async fn handle_stream(&self, request: &Request, stream: Stream<'_>) -> Result<bool, Error> {
        Ok(request.keep_alive())
    }

    /// Internal function which implements the bulk of Rwf controller logic. Do not implement this unless
    /// you're looking to do something entirely different.
    ///
    /// Things handled by this method:
    ///
    /// - Checking authentication
    /// - Running middleware
    /// - Ensuring each request has a session
    ///
    /// Controllers that override this need to be aware of the internal functionality of Rwf
    /// and act accordingly.
    async fn handle_internal(&self, request: Request) -> Result<Response, Error> {
        let auth = self.auth();

        if !auth.auth().authorize(&request).await? {
            return auth.auth().denied(&request).await;
        }

        let request = request.set_skip_csrf(self.skip_csrf());

        // Run the middleware chain (forward).
        let outcome = self.middleware().handle_request(request).await?;

        let response = match outcome {
            (Outcome::Forward(request), executed) => match self.handle(&request).await {
                Ok(response) => {
                    self.middleware()
                        .handle_response(&request, response.from_request(&request)?, executed)
                        .await?
                }
                Err(err) => {
                    error!("{:?}", err);

                    let response = match err {
                        Error::HttpError(err) => match err.code() {
                            400 => Response::bad_request(),
                            401 => Response::unauthorized(None),
                            413 => Response::content_too_large(),
                            _ => Response::internal_error(err),
                        },

                        Error::ViewError(err) => {
                            Response::error_pretty("Template error", err.to_string().as_str())
                        }

                        err => Response::internal_error(err),
                    };

                    // Run the middleware chain on the response anyway.
                    self.middleware()
                        .handle_response(&request, response, executed)
                        .await?
                }
            },
            (Outcome::Stop(request, response), executed) => {
                self.middleware()
                    .handle_response(&request, response.from_request(&request)?, executed)
                    .await?
            }
        };

        Ok(response)
    }

    /// Handle the request and return a response. Implement this function to define how your controller
    /// will respond to requests.
    /// This method is asynchronous, and since we use `async_trait`, the signature can be a bit confusing.
    /// The actual method is:
    ///
    /// ```rust,ignore
    /// async fn handle(&self, request: &Request) -> Result<Response, Error>;
    /// ```
    async fn handle(&self, request: &Request) -> Result<Response, Error>;

    /// The name of this controller. Used for logging. All names are globally unique, so
    /// you won't need to override this method.
    fn controller_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// A controller that splits GET and POST requests into two different methods.
///
/// Most web apps using templates would want to implement the `PageController` which splits up `GET` from `POST` requests,
/// allowing to handle form submissions together with page rendering.
///
/// #### Example
///
/// ```
/// // Include all necessary types.
/// use rwf::prelude::*;
///
/// // Your controller.
/// #[derive(Default, macros::PageController)]
/// pub struct MyPage;
///
/// // Controllers are async and use the `async_trait` crate.
/// #[rwf::async_trait]
/// impl PageController for MyPage {
///     // Respond to a GET request.
///     async fn get(&self, request: &Request) -> Result<Response, Error> {
///         render!(request, "templates/my_page.html")
///     }
/// }
/// ```
///
/// The `macros::PageController` expands to this:
///
/// ```rust,ignore
/// #[rwf::async_trait]
/// impl Controller for MyPage {
///     async fn handle(&self, request: &Request) -> Result<Response, Error> {
///         // Delegate request handling to the `PageController::handle` method.
///         PageController::handle(self, request)
///     }
/// }
/// ```
///
/// This is required because of how Rust trait dynamic dispatch works. Rwf HTTP server only handles structs that implement the [`Controller`] trait, so all controllers that implement
/// a descendant trait (e.g. [`PageController`]) must also implement the supertrait ([`Controller`]).
#[async_trait]
#[allow(unused_variables)]
pub trait PageController: Controller {
    /// Respond to a `GET` request to this controller.
    async fn get(&self, request: &Request) -> Result<Response, Error>;

    /// Respond to a `POST` request to this controller.
    /// By default, `405 - Method Not Allowed` is returned.
    async fn post(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::method_not_allowed())
    }

    /// Perform the request GET/POST split automatically.
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        if request.get() {
            PageController::get(self, request).await
        } else if request.post() {
            PageController::post(self, request).await
        } else {
            Ok(Response::method_not_allowed())
        }
    }
}

/// REST controller, which splits requests into 6 REST verbs.
///
/// This controller will split incoming requests based on the REST specification and route
/// them to their respective methods.
///
/// Available methods are:
///
/// - list (`GET /path`)
/// - create (`POST /path`)
/// - get (`GET /path/:id`)
/// - update (`PUT /path/:id`)
/// - patch (`PATCH /path/:id`)
/// - delete (`DELETE /path/:id`)
///
/// By default, all methods will respond with `501 - Not Implemented`. It's up to the user
/// to implement each method according to their needs.
///
/// The `:id` can be any value which implements the [`ToParameter`] trait.
/// Common data types are implemented, e.g., [`i64`] and [`String`].
///
/// ### Example
///
/// ```
/// use rwf::prelude::*;
///
/// #[derive(Default, macros::RestController)]
/// struct MyController;
///
/// #[async_trait]
/// impl RestController for MyController {
///     type Resource = i64;
///
///     async fn get(&self, request: &Request, id: &i64) -> Result<Response, Error> {
///         Ok(Response::new().html(format!("Hello, id #{}", id)))
///     }
/// }
/// ```
#[async_trait]
#[allow(unused_variables)] // Easier to read the code without _var_name.
pub trait RestController: Controller {
    /// Resource type used in the request.
    ///
    /// Rust is a typed language, this makes handling IDs easier by specifying the
    /// expected data type. Inputs not matching this data type will be rejected.
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

    /// Get a route handler for this controller. Used when
    /// adding this controller to the server with a route mapping.
    ///
    /// Use the `rest!` macro instead, e.g.:
    ///
    /// ```ignore
    /// rest!("/path" => YourResetController)
    /// ```
    fn rest(self, path: &str) -> Handler
    where
        Self: Sized + 'static,
    {
        Handler::rest(path, self)
    }

    /// Responds to `GET /path`. List all available resources at this endpoint.
    /// Pagination is allowed.
    ///
    /// # Signature
    ///
    /// ```ignore
    /// async fn list(&self, request: &Request) -> Result<Response, Error>;
    /// ```
    async fn list(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::method_not_allowed())
    }

    /// Responds to `GET /path/:id`. Fetch a specific resource, as identified
    /// in the request.
    ///
    /// # Signature
    ///
    /// ```ignore
    /// async fn get(&self, request: &Request, id: &Self::Resource) -> Result<Response, Error>;
    /// ```
    async fn get(&self, request: &Request, id: &Self::Resource) -> Result<Response, Error> {
        Ok(Response::method_not_allowed())
    }

    /// Responds to `POST /path`. Create a new resource.
    ///
    /// # Signature
    ///
    /// ```ignore
    /// async fn create(&self, request: &Request) -> Result<Response, Error>;
    /// ```
    async fn create(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::method_not_allowed())
    }

    /// Responds to `PUT /path/:id`. Update an existing resource.
    ///
    /// # Signature
    ///
    /// ```ignore
    /// async fn update(&self, request: &Request, id: &Self::Resource) -> Result<Response, Error>;
    /// ```
    async fn update(&self, request: &Request, id: &Self::Resource) -> Result<Response, Error> {
        Ok(Response::method_not_allowed())
    }

    /// Responds to `PATCH /path/:id`. Partially update an existing resource.
    ///
    /// # Signature
    ///
    /// ```ignore
    /// async fn patch(&self, request: &Request, id: &Self::Resource) -> Result<Response, Error>;
    /// ```
    async fn patch(&self, request: &Request, id: &Self::Resource) -> Result<Response, Error> {
        Ok(Response::method_not_allowed())
    }

    /// Responds to `DELETE /path:id`. Deletes an existing resource.
    ///
    /// # Signature
    ///
    /// ```ignore
    /// async fn delete(&self, request: &Request, id: &Self::Resource) -> Result<Response, Error>;
    /// ```
    async fn delete(&self, request: &Request, id: &Self::Resource) -> Result<Response, Error> {
        Ok(Response::method_not_allowed())
    }
}

/// A controller that extends the [`RestController`] to
/// automatically performs CRUD actions on database models.
///
/// # Example
///
/// ```
/// # use rwf::prelude::*;
/// use serde::{Serialize, Deserialize};
///
/// // The database model.
/// #[derive(Clone, macros::Model, Serialize, Deserialize)]
/// struct User {
///     id: Option<i64>,
///     email: String,
/// }
///
/// // The controller.
/// #[derive(macros::ModelController)]
/// struct UserController;
///
/// #[rwf::async_trait]
/// impl ModelController for UserController {
///     type Model = User;
/// }
/// ```
#[async_trait]
pub trait ModelController: Controller {
    /// The database model.
    type Model: Model + Serialize + Send + Sync + for<'a> Deserialize<'a>;

    /// Handle the request to this controller.
    async fn handle(&self, request: &Request) -> Result<Response, Error> {
        let method = request.method();
        let parameter = request.parameter::<i64>("id");

        match parameter {
            Ok(Some(id)) => match method {
                Method::Get => ModelController::get(self, request, &id).await,
                Method::Put => ModelController::update(self, request, &id).await,
                Method::Delete => ModelController::delete(self, request, &id).await,
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

    /// Returns the controller route handler. Used when mapping this
    /// controller to a path in the server.
    ///
    /// Use `crud!` instead:
    ///
    /// ```ignore
    /// crud!("/path" => MyController)
    /// ```
    fn crud(self, path: &str) -> Handler
    where
        Self: Sized + 'static,
    {
        Handler::rest(path, self)
    }

    /// List all records for the model. Supports pagination with `page` parameter. Supports number of records per page with `page_size` parameter.
    ///
    /// # Example
    ///
    /// ```text,ignore
    /// GET /users?page=3&page_size=40
    /// ```
    async fn list(&self, request: &Request) -> Result<Response, Error> {
        let mut conn = get_connection().await?;
        let page_size = request.query().get::<i64>("page_size").unwrap_or(25);
        let page = request.query().get::<i64>("page").unwrap_or(1);
        let offset = (std::cmp::max(1, page) - 1) * page_size;

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

    /// Fetch a model record identified by its primary key.
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

    /// Create new model record.
    async fn create(&self, request: &Request) -> Result<Response, Error> {
        let model = match request.json::<Self::Model>() {
            Ok(model) => model,
            Err(err) => {
                println!("ser err: {:?}", err);
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

        Ok(Response::new().code(201).json(model)?)
    }

    /// Update existing model record.
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

    async fn delete(&self, _request: &Request, id: &i64) -> Result<Response, Error> {
        let mut conn = get_connection().await?;

        match Self::Model::find_by(Self::Model::primary_key(), *id)
            .fetch_optional(&mut conn)
            .await
        {
            Ok(Some(model)) => {
                model.destroy().fetch(&mut conn).await?;
                match Response::new().json(model) {
                    Ok(response) => Ok(response),
                    Err(err) => Ok(Response::internal_error(err)),
                }
            }
            Ok(None) => Ok(Response::not_found()),
            Err(e) => Ok(Response::internal_error(e)),
        }
    }

    /// Partially update an existing model record.
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

/// A controller that handles WebSocket connections.
#[async_trait]
#[allow(unused_variables)]
pub trait WebsocketController: Controller {
    /// Handle WebSocket connection.
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

    /// Handle an incoming client message.
    async fn client_message(
        &self,
        session_id: &SessionId,
        message: websocket::Message,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// Do something when a client creates a new WebSocket connection.
    async fn client_connected(&self, session_id: &SessionId) -> Result<(), Error> {
        Ok(())
    }

    /// Handle the WebSocket TCP stream. Provides the WebSocket
    /// protocol implementation. You may not want to override this unless you
    /// want to change how WebSockets work in Rwf.
    async fn handle_stream(
        &self,
        request: &Request,
        mut stream: Stream<'_>,
    ) -> Result<bool, Error> {
        use tokio::sync::broadcast::error::RecvError;

        let session_id = request.session().session_id.clone();

        info!(
            "{} {} {} connected",
            "websocket".purple(),
            request.path().path().purple(),
            self.controller_name().green(),
        );

        let config = get_config();
        //let mut stream = stream;
        let mut receiver = Comms::receiver(&session_id);
        let mut check = interval(config.websocket.ping_interval().unsigned_abs());
        let mut lost_pings = 0_i64;

        self.client_connected(&session_id).await?;

        loop {
            select! {
                _ = check.tick() => {
                    debug!("{} check session \"{}\"", "websocket".purple(), session_id);

                    let closed = match timeout(
                        config.websocket.ping_timeout().unsigned_abs(),
                        DataFrame::new_ping().flush(&mut stream)
                    ).await {
                        Ok(Ok(_)) => false,
                        _ => true,
                    };

                    lost_pings += 1;

                    if closed || lost_pings as usize > config.websocket.ping_disconnect_count {
                        break;
                    }
                }

                message = receiver.recv() => {
                    match message {
                        Ok(message) => {
                            debug!("{} sending {:?} to session \"{}\"",
                                "websocket".purple(),
                                message, receiver.session_id());
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
                        debug!("{} session \"{}\" is alive", "websocket".purple(), session_id);
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
