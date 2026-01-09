//! Authentication system.
//!
//! Made to be easily extendable. Users need only to implement the [`Authentication`] trait
//! and set it on their controller. Rwf also comes with several built-in authentication mechanisms that
//! can be used out of the box.
use super::Error;
use crate::comms::WebsocketSender;
use crate::config::get_config;
use crate::http::{Authorization, Request, Response};
use crate::view::{ToTemplateValue, Value};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use utoipa::openapi;
use utoipa::openapi::{OpenApi, SecurityRequirement};

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

impl utoipa::Modify for AuthHandler {
    fn modify(&self, openapi: &mut OpenApi) {
        self.auth().modify(openapi);
        let unauthorized_response = utoipa::openapi::Response::new("An Unauthorized access attempted");
        for path in openapi.paths.paths.values_mut() {
            for operation in [&mut path.get, &mut path.post, &mut path.put, &mut path.patch, &mut path.head] {
                if let Some(ref mut op) = operation {
                    op.responses.responses.entry("401".to_string()).or_insert(utoipa::openapi::RefOr::T(unauthorized_response.clone()));
                }
            }
        }
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
pub trait Authentication: Sync + Send + utoipa::Modify {
    /// Perform the authentication and allow or deny the request from
    /// going forward.
    async fn authorize(&self, request: &Request) -> Result<bool, Error>;

    /// If the request is denied, return a specific response.
    /// Default is `401 - Unauthorized`.
    async fn denied(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::unauthorized(None))
    }

    /// Returns an authentication handler used when configuring
    /// authentication on a controller.
    fn handler(self) -> AuthHandler
    where
        Self: Sized + 'static,
    {
        AuthHandler::new(self)
    }
}

/// Allow all requests. This is the default authentication method for all controllers.
pub struct AllowAll;

impl utoipa::Modify for AllowAll {
    fn modify(&self, openapi: &mut OpenApi) {
        if let Some(ref mut sec) = openapi.security {
            sec.push(SecurityRequirement::default());
        } else {
            openapi.security = Some(vec![SecurityRequirement::default()]);
        }
    }
}

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

impl utoipa::Modify for DenyAll {
    fn modify(&self, openapi: &mut OpenApi) {
        let scopes: Vec<String> = Vec::new();
        let requirement = SecurityRequirement::new("not_existent_security_scheme", scopes);
        if let Some(ref mut sec) = openapi.security {
            sec.push(requirement)
        } else {
            openapi.security = Some(vec![requirement]);
        }
    }
}

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
        Ok(Response::unauthorized(Some("Basic")))
    }
}

impl utoipa::Modify for BasicAuth {
    fn modify(&self, openapi: &mut OpenApi) {
        if let Some(ref mut components) = openapi.components {
            components
                .add_security_scheme(
                    "http_basic_auth",
                    openapi::security::SecurityScheme::Http(
                        openapi::security::HttpBuilder::new()
                            .scheme(
                                openapi::security::HttpAuthScheme::Basic
                            )
                            .description(
                                Some("A Path protected by a HTTP Basic AUth middleware")
                            )
                            .build()
                    )
                )
        }
        let scopes: Vec<String> = Vec::new();
        let requirement = SecurityRequirement::new("http_basic_auth", scopes);
        if openapi.security.is_none() {
            openapi.security = Some(vec![requirement.clone()]);
        } else {
            openapi.security.as_mut().unwrap().push(requirement.clone());
        }
        for path in &mut openapi.paths.paths {
            for operation in [&mut path.1.get, &mut path.1.post, &mut path.1.delete, &mut path.1.patch, &mut path.1.put].into_iter() {
                if let Some(ref mut op) = operation {
                    if let Some(ref mut sec) = op.security {
                        sec.push(requirement.clone())
                    } else {
                        op.security = Some(vec![requirement.clone()]);
                    }
                }
            }
        }
    }
}

/// Static token authentication (basically a passphrase).
///
/// Not very secure since the token can leak, but helpful if you need
/// to quickly protect an endpoint.
pub struct Token {
    /// A token string.
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

impl utoipa::Modify for Token {
    fn modify(&self, openapi: &mut OpenApi) {
        let scopes: Vec<String> = Vec::new();
        let requirement = SecurityRequirement::new("token_auth", scopes);
        if let Some(ref mut sec) = openapi.security {
            sec.push(requirement.clone())
        } else {
            openapi.security = Some(vec![requirement.clone()]);
        }

        if let Some(ref mut components) = openapi.components {
            let mut token_header = openapi::security::ApiKeyValue::new("Authorization:");
            token_header.description = Some("A Authorization Header holdig a Token. The Value must begin with 'Token'".to_string());
            components.add_security_scheme(
                "token_auth",
                openapi::security::SecurityScheme::ApiKey(
                    openapi::security::ApiKey::Header(token_header)
                )
            )
        }
        for path in &mut openapi.paths.paths {
            for operation in [&mut path.1.get, &mut path.1.post, &mut path.1.delete, &mut path.1.patch, &mut path.1.put].into_iter() {
                if let Some(ref mut op) = operation {
                    if let Some(ref mut sec) = op.security {
                        sec.push(requirement.clone())
                    } else {
                        op.security = Some(vec![requirement.clone()]);
                    }
                }
            }
        }


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

impl ToTemplateValue for Session {
    fn to_template_value(&self) -> Result<Value, crate::view::Error> {
        let mut hash = HashMap::new();
        hash.insert("expiration".into(), Value::Integer(self.expiration));
        hash.insert(
            "session_id".into(),
            Value::String(self.session_id.to_string()),
        );
        hash.insert(
            "payload".into(),
            Value::String(serde_json::to_string(&self.payload).unwrap()),
        );

        Ok(Value::Hash(hash))
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

    /// The session is close to being expired and should be renewed automatically.
    pub fn should_renew(&self) -> bool {
        if let Ok(expiration) = OffsetDateTime::from_unix_timestamp(self.expiration) {
            let now = OffsetDateTime::now_utc();
            let remains = expiration - now;
            let session_duration = get_config().general.session_duration();
            remains < session_duration / 2 && remains.is_positive() // not expired
        } else {
            true
        }
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
    /// of just returning `401 - Unauthorized`.
    pub fn redirect(url: impl ToString) -> Self {
        Self {
            redirect: Some(url.to_string()),
        }
    }
}

impl utoipa::Modify for SessionAuth {
    fn modify(&self, openapi: &mut OpenApi) {
        let scopes: Vec<String> = Vec::new();
        let requirement = SecurityRequirement::new("session_auth", scopes);
        if let Some(ref mut sec) = openapi.security {
            sec.push(requirement.clone());
        } else  {
            openapi.security = Some(vec![requirement.clone()]);
        }

        if let Some(ref mut components) = openapi.components {
            components.add_security_scheme("session_auth",
                                           openapi::security::SecurityScheme::ApiKey(
                                               openapi::security::ApiKey::Cookie(
                                                   openapi::security::ApiKeyValue::new("rwf_session")
                                               )
                                           )
            );
        }
        for path in &mut openapi.paths.paths {
            for operation in [&mut path.1.get, &mut path.1.post, &mut path.1.delete, &mut path.1.patch, &mut path.1.put].into_iter() {
                if let Some(ref mut op) = operation {
                    if let Some(ref mut sec) = op.security {
                        sec.push(requirement.clone())
                    } else {
                        op.security = Some(vec![requirement.clone()]);
                    }
                }
            }
        }
    }
}

#[async_trait]
impl Authentication for SessionAuth {
    async fn authorize(&self, request: &Request) -> Result<bool, Error> {
        Ok(request.session().authenticated())
    }

    async fn denied(&self, _request: &Request) -> Result<Response, Error> {
        if let Some(ref redirect) = self.redirect {
            Ok(Response::new().redirect(redirect))
        } else {
            Ok(Response::unauthorized(None))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_should_renew() {
        let mut session = Session::default();
        assert!(!session.should_renew());

        assert_eq!(get_config().general.session_duration(), Duration::weeks(4));

        session.expiration = (OffsetDateTime::now_utc() + Duration::weeks(2)
            - Duration::seconds(5))
        .unix_timestamp();
        assert!(session.should_renew());

        session.expiration =
            (OffsetDateTime::now_utc() + Duration::weeks(2) + Duration::seconds(5))
                .unix_timestamp();
        assert!(!session.should_renew());
    }
}
