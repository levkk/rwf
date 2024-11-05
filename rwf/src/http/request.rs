//! HTTP request.

use std::fmt::Debug;
use std::marker::Unpin;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;

use serde::Deserialize;
use serde_json::{Deserializer, Value};
use time::OffsetDateTime;
use tokio::io::{AsyncRead, AsyncReadExt};

use super::{Cookies, Error, FormData, FromFormData, Head, Params, Response, ToParameter};
use crate::{
    controller::{Session, SessionId},
    model::{ConnectionGuard, Model},
};

/// HTTP request.
///
/// The request is fully loaded into memory. It's safe to clone
/// since the contents are behind an [`std::sync::Arc`].
#[derive(Debug, Clone)]
pub struct Request {
    head: Head,
    session: Option<Session>,
    inner: Arc<Inner>,
    params: Option<Arc<Params>>,
    received_at: OffsetDateTime,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            head: Head::default(),
            session: None,
            inner: Arc::new(Inner::default()),
            params: None,
            received_at: OffsetDateTime::now_utc(),
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Inner {
    body: Vec<u8>,
    cookies: Cookies,
    peer: Option<SocketAddr>,
}

impl Request {
    /// Read the request in its entirety from a stream.
    pub async fn read(peer: SocketAddr, mut stream: impl AsyncRead + Unpin) -> Result<Self, Error> {
        let head = Head::read(&mut stream).await?;
        let content_length = head.content_length().unwrap_or(0);
        let mut body = vec![0u8; content_length];
        stream
            .read_exact(&mut body)
            .await
            .map_err(|_| Error::MalformedRequest("incorrect content length"))?;

        let cookies = head.cookies();

        Ok(Request {
            head,
            params: None,
            session: cookies.get_session()?,
            inner: Arc::new(Inner {
                body,
                peer: Some(peer),
                cookies,
            }),
            received_at: OffsetDateTime::now_utc(),
        })
    }

    /// Get the request's source IP address.
    pub fn peer(&self) -> &SocketAddr {
        self.inner
            .peer
            .as_ref()
            .expect("peer is not set on the request")
    }

    /// Set params on the request.
    pub fn with_params(mut self, params: Arc<Params>) -> Self {
        self.params = Some(params);
        self
    }

    pub fn head(&self) -> &Head {
        &self.head
    }

    pub fn head_mut(&mut self) -> &mut Head {
        &mut self.head
    }

    /// Extract a parameter from the provided path.
    pub fn parameter<T: ToParameter>(&self, name: &str) -> Result<Option<T>, Error> {
        if let Some(ref params) = self.params {
            if let Some(parameter) = params.parameter(self.path().base(), name) {
                return Ok(Some(T::to_parameter(&parameter)?));
            }
        }

        Ok(None)
    }

    /// Request's body as bytes.
    ///
    /// It's the job of the caller to handle encoding if any.
    pub fn body(&self) -> &[u8] {
        &self.inner.body
    }

    /// Request's body as JSON value.
    pub fn json_raw(&self) -> Result<Value, serde_json::Error> {
        self.json()
    }

    /// Request's body as a UTF-8 string.
    /// UTF-8 encoding is assumed, and all incompatible characters are dropped.
    pub fn string(&self) -> String {
        String::from_utf8_lossy(self.body()).to_string()
    }

    pub fn form_data(&self) -> Result<FormData, Error> {
        FormData::from_request(self)
    }

    pub fn form<T: FromFormData>(&self) -> Result<T, Error> {
        T::from_form_data(&self.form_data()?)
    }

    /// Request's body deserialized from JSON into a particular Rust type.
    pub fn json<'a, T: Deserialize<'a>>(&'a self) -> Result<T, serde_json::Error> {
        let mut deserializer = Deserializer::from_slice(self.body());
        T::deserialize(&mut deserializer)
    }

    /// Request's cookies.
    pub fn cookies(&self) -> &Cookies {
        &self.inner.cookies
    }

    /// Request's session.
    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }

    /// When thre request was received.
    pub fn received_at(&self) -> OffsetDateTime {
        self.received_at
    }

    /// Get the session identifier.
    ///
    /// This will be a random string if it's a guest
    /// or a unique integer if logged in.
    pub fn session_id(&self) -> Option<SessionId> {
        self.session
            .as_ref()
            .map(|session| session.session_id.clone())
    }

    /// Get the authenticated user's ID. Return HTTP 403
    /// if not logged in.
    pub fn user_id(&self) -> Result<i64, Error> {
        if let Some(session_id) = self.session_id() {
            match session_id {
                SessionId::Authenticated(id) => Ok(id),
                _ => Err(Error::Forbidden),
            }
        } else {
            Err(Error::Forbidden)
        }
    }

    /// Fetch the user that's currently authenticated, if any.
    pub async fn user<T: Model>(&self, conn: &mut ConnectionGuard) -> Result<Option<T>, Error> {
        match self.session_id() {
            Some(SessionId::Authenticated(user_id)) => {
                Ok(Some(T::find(user_id).fetch(conn).await?))
            }

            _ => Ok(None),
        }
    }

    /// Fetch the user that's currently authenticated. If none, return HTTP 403.
    pub async fn user_required<T: Model>(&self, conn: &mut ConnectionGuard) -> Result<T, Error> {
        match self.user(conn).await? {
            Some(user) => Ok(user),
            None => Err(Error::Forbidden),
        }
    }

    /// Set the session on the request.
    ///
    /// For internal use only. This is automatically done by the HTTP server,
    /// if the session is available.
    pub fn set_session(mut self, session: Option<Session>) -> Self {
        self.session = session;
        self
    }

    /// Is the client requesting upgrade to use WebSocket?
    pub fn upgrade_websocket(&self) -> bool {
        self.headers()
            .get("connection")
            .map(|v| v.to_lowercase().contains("upgrade"))
            == Some(true)
            && self.headers().get("upgrade").map(|v| v.to_lowercase())
                == Some(String::from("websocket"))
    }

    /// Create an authenticated session for the provided user identifier.
    pub fn login(&self, user_id: i64) -> Response {
        let mut session = self
            .session()
            .map(|s| s.clone())
            .unwrap_or(Session::empty());
        session.session_id = SessionId::Authenticated(user_id);
        Response::new().set_session(session).html("")
    }

    /// Log the user out.
    ///
    /// This overwrites the session cookie with a guest session.
    pub fn logout(&self) -> Response {
        let mut session = self
            .session()
            .map(|s| s.clone())
            .unwrap_or(Session::empty());
        session.session_id = SessionId::default();
        Response::new().set_session(session).html("")
    }
}

impl Deref for Request {
    type Target = Head;

    fn deref(&self) -> &Self::Target {
        &self.head
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    pub async fn dummy_request() -> Result<Request, Error> {
        let body = ("GET /?hello=world HTTP/1.1\r\n".to_owned()
            + "Content-Type: application/json\r\n"
            + "Accept: */*\r\n"
            + "Content-Length: 18\r\n"
            + "\r\n"
            + r#"{"hello": "world"}"#)
            .as_bytes()
            .to_vec();

        let req = Request::read("127.0.0.1:1337".parse().unwrap(), &body[..]).await?;

        Ok(req)
    }

    #[tokio::test]
    async fn test_response() {
        #[derive(Deserialize)]
        struct Hello {
            hello: String,
        }

        let request = dummy_request().await.unwrap();
        let json = request.json::<Hello>().expect("deserialize body");
        assert_eq!(json.hello, "world");
    }
}
