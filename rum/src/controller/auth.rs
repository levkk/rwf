//! Authentication system.
//!
//! Made to be easily extendable. Users need only to implement the [`crate::controller::auth::Authentication`] trait
//! and set it on their controller.
use super::Error;
use crate::http::{Authorization, Request, Response};

use async_trait::async_trait;

/// Authenticators need to implement this trait.
#[async_trait]
#[allow(unused_variables)]
pub trait Authentication: Sync + Send {
    async fn authorize(&self, request: &Request) -> Result<bool, Error>;
    async fn denied(&self, request: &Request) -> Result<Response, Error> {
        Ok(Response::not_authorized())
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
