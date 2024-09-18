use super::Error;
use crate::http::Request;

use async_trait::async_trait;

#[async_trait]
pub trait Authentication: Sync + Send {
    async fn authorize(&self, request: &Request) -> Result<bool, Error>;
}

pub struct AllowAll;

#[async_trait]
impl Authentication for AllowAll {
    async fn authorize(&self, _request: &Request) -> Result<bool, Error> {
        Ok(true)
    }
}

pub struct DenyAll;

#[async_trait]
impl Authentication for DenyAll {
    async fn authorize(&self, _request: &Request) -> Result<bool, Error> {
        Ok(false)
    }
}

pub struct BasicAuth {
    pub user: String,
    pub password: String,
}

pub struct Token {
    pub token: String,
}
