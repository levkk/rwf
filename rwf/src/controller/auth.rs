//! Authentication system.
//!
//! Made to be easily extendable. Users need only to implement the [`Authentication`] trait
//! and set it on their controller. Rwf also comes with several built-in authentication mechanisms that
//! can be used out of the box.
use super::Error;
use crate::comms::WebsocketSender;
use crate::config::get_config;
use crate::http::{Authorization, Request, Response};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use std::fmt::Debug;
use std::sync::Arc;

/// An authentication mechanism wrapper that can be attached to a controller.
#[derive(Clone)]
pub struct AuthHandler {
    auth: Arc<Box<dyn Authentication>>,
}

impl Default for AuthHandler {
    fn default() -> Self {
        Self::new(AllowAll {})
    }
}

impl AuthHandler {
    /// Create new authentication mechanism using the provided authentication method.
    pub fn new(auth: impl Authentication + 'static) -> Self {
        AuthHandler {
            auth: Arc::new(Box::new(auth)),
        }
    }

    /// Get the authentication method.
    pub fn auth(&self) -> &Box<dyn Authentication> {
        &self.auth
    }
}

/// Authenticators need to implement this trait.
#[async_trait]
#[allow(unused_variables)]
pub trait Authentication: Sync + Send {
    /// Perform the authentication and allow or deny the request from
    /// going forward.
    async fn authorize(&self, request: &Request) -> Result<bool, Error>;

    /// If the request is denied, return a specific response.
    /// Default is `403 - Forbidden`.
    async fn denied(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::forbidden())
    }

    fn handler(self) -> AuthHandler
    where
        Self: Sized + 'static,
    {
        AuthHandler::new(self)
    }
}

/// Allow all requests. This is the default authentication method for all controllers.
pub struct AllowAll;

#[async_trait]
impl Authentication for AllowAll {
    async fn authorize(&self, _request: &Request) -> Result<bool, Error> {
        Ok(true)
    }
}

/// Deny all requests.
///
/// Not particularly useful, since there is no way to override it,
/// but it is included to demonstrate how authentication works.
pub struct DenyAll;

#[async_trait]
impl Authentication for DenyAll {
    async fn authorize(&self, _request: &Request) -> Result<bool, Error> {
        Ok(false)
    }
}

/// HTTP Basic authentication.
pub struct BasicAuth {
    /// Username.
    pub user: String,
    /// Password.
    pub password: String,
}

#[async_trait]
impl Authentication for BasicAuth {
    async fn authorize(&self, request: &Request) -> Result<bool, Error> {
        Ok(
            if let Some(Authorization::Basic { user, password }) = request.authorization() {
                self.user == user && self.password == password
            } else {
                false
            },
        )
    }

    async fn denied(&self, _request: &Request) -> Result<Response, Error> {
        Ok(Response::unauthorized("Basic"))
    }
}

/// Static token authentication (basically a passphrase).
///
/// Not very secure since the token can leak, but helpful if you need
/// to quickly protect an endpoint.
pub struct Token {
    pub token: String,
}

#[async_trait]
impl Authentication for Token {
    async fn authorize(&self, request: &Request) -> Result<bool, Error> {
        Ok(
            if let Some(Authorization::Token { token }) = request.authorization() {
                self.token == token
            } else {
                false
            },
        )
    }
}

/// Type of session provided by the client in the request.
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum SessionId {
    /// Guest user. All visitors are given a guest session.
    Guest(String),
    /// Authenticated user. This user has passed an authentication challenge, e.g. username and password.
    Authenticated(i64),
}

impl SessionId {
    /// The session is authenticated, i.e. it's a user.
    pub fn authenticated(&self) -> bool {
        use SessionId::*;

        match self {
            Guest(_) => false,
            Authenticated(_) => true,
        }
    }

    /// The session is a guest session, i.e. anonymous, not logged in.
    pub fn guest(&self) -> bool {
        !self.authenticated()
    }

    /// Get the user's ID. This is an arbitrary integer, but
    /// should ideally be the primary key of a `"users"` table, if such exists.
    pub fn user_id(&self) -> Option<i64> {
        match self {
            SessionId::Authenticated(id) => Some(*id),
            _ => None,
        }
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionId::Authenticated(id) => write!(f, "{}", id),
            SessionId::Guest(id) => write!(f, "{}", id),
        }
    }
}

impl Default for SessionId {
    fn default() -> Self {
        use rand::{distributions::Alphanumeric, thread_rng, Rng};

        SessionId::Guest(
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect::<String>(),
        )
    }
}

/// A client's session.
///
/// This is a JSON-encoded object
/// that's stored securely in a cookie (using encryption).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Session {
    /// Customizable session payload.
    #[serde(rename = "p")]
    pub payload: serde_json::Value,
    /// Session expiration (UNIX timestamp in UTC).
    #[serde(rename = "e")]
    pub expiration: i64,
    /// Type of session, e.g. guest or user.
    #[serde(rename = "s")]
    pub session_id: SessionId,
}

impl Default for Session {
    fn default() -> Self {
        Self::new(serde_json::json!({})).expect("json")
    }
}

impl Session {
    /// Create a guest session.
    pub fn anonymous() -> Self {
        Self::default()
    }

    /// Alias for creating a guest session.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create new session with this payload. This creates a guest session.
    pub fn new(payload: impl Serialize) -> Result<Self, Error> {
        Ok(Self {
            payload: serde_json::to_value(payload)?,
            expiration: (OffsetDateTime::now_utc() + get_config().general.session_duration())
                .unix_timestamp(),
            session_id: SessionId::default(),
        })
    }

    /// Create new session with this payload, authenticated to a particular user.
    pub fn new_authenticated(payload: impl Serialize, user_id: i64) -> Result<Self, Error> {
        let mut session = Self::new(payload)?;
        session.session_id = SessionId::Authenticated(user_id);

        Ok(session)
    }

    /// Renew the session for the specified duration.
    pub fn renew(mut self, renew_for: Duration) -> Self {
        self.expiration = (OffsetDateTime::now_utc() + renew_for).unix_timestamp();
        self
    }

    /// Check if the session has expired.
    pub fn expired(&self) -> bool {
        if let Ok(expiration) = OffsetDateTime::from_unix_timestamp(self.expiration) {
            let now = OffsetDateTime::now_utc();
            expiration < now
        } else {
            false
        }
    }

    /// Get a Websocket sender for this session. This allows to send arbitray messages
    /// to all browsers connected with this session.
    pub fn websocket(&self) -> WebsocketSender {
        use crate::comms::Comms;
        Comms::websocket(&self.session_id)
    }

    /// This session is authenticated to a user and hasn't expired.
    pub fn authenticated(&self) -> bool {
        !self.expired() && self.session_id.authenticated()
    }

    /// This is a guest session.
    pub fn guest(&self) -> bool {
        !self.expired() && self.session_id.guest()
    }
}

/// Session authentication.
#[derive(Default)]
pub struct SessionAuth {
    redirect: Option<String>,
}

impl SessionAuth {
    /// Create session authentication which redirects to this URL instead
    /// of just returning `403 - Unauthorized`.
    pub fn redirect(url: impl ToString) -> Self {
        Self {
            redirect: Some(url.to_string()),
        }
    }
}

#[async_trait]
impl Authentication for SessionAuth {
    async fn authorize(&self, request: &Request) -> Result<bool, Error> {
        if let Some(session) = request.session() {
            Ok(session.authenticated())
        } else {
            Ok(false)
        }
    }

    async fn denied(&self, _request: &Request) -> Result<Response, Error> {
        if let Some(ref redirect) = self.redirect {
            Ok(Response::new().redirect(redirect))
        } else {
            Ok(Response::forbidden())
        }
    }
}
