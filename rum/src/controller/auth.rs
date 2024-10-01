//! Authentication system.
//!
//! Made to be easily extendable. Users need only to implement the [`crate::controller::auth::Authentication`] trait
//! and set it on their controller.
use super::Error;
use crate::config::get_config;
use crate::http::{Authorization, Request, Response};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

use std::fmt::Debug;
use std::sync::Arc;

/// An authentication mechanism that can be attached to a controller.
#[derive(Clone)]
pub struct AuthHandler {
    auth: Arc<Box<dyn Authentication>>,
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
    /// Default is 403 - Forbidden.
    async fn denied(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::forbidden())
    }
}

/// Allow all requests.
pub struct AllowAll;

#[async_trait]
impl Authentication for AllowAll {
    async fn authorize(&self, _request: &Request) -> Result<bool, Error> {
        Ok(true)
    }
}

/// Deny all requests.
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UserId {
    Guest(String),
    Authenticated(i64),
}

impl Default for UserId {
    fn default() -> Self {
        use rand::{distributions::Alphanumeric, thread_rng, Rng};

        UserId::Guest(
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect::<String>(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    #[serde(rename = "p")]
    pub payload: serde_json::Value,
    #[serde(rename = "e")]
    pub expiration: i64,
    #[serde(rename = "u")]
    pub user_id: UserId,
}

impl Session {
    pub fn new(payload: impl Serialize) -> Result<Self, Error> {
        Ok(Self {
            payload: serde_json::to_value(payload)?,
            expiration: (OffsetDateTime::now_utc() + get_config().session_duration)
                .unix_timestamp(),
            user_id: UserId::default(),
        })
    }

    pub fn renew(mut self, renew_for: Duration) -> Self {
        self.expiration = (OffsetDateTime::now_utc() + renew_for).unix_timestamp();
        self
    }

    pub fn expired(&self) -> bool {
        if let Ok(expiration) = OffsetDateTime::from_unix_timestamp(self.expiration) {
            let now = OffsetDateTime::now_utc();
            expiration > now
        } else {
            false
        }
    }
}

#[async_trait]
impl Authentication for Session {
    async fn authorize(&self, request: &Request) -> Result<bool, Error> {
        if let Some(session) = request.session() {
            Ok(!session.expired())
        } else {
            Ok(false)
        }
    }
}
