//! HTTP request.
use std::marker::Unpin;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
use std::{collections::HashMap, fmt::Debug};

use serde::Deserialize;
use serde_json::{Deserializer, Value};
use time::OffsetDateTime;
use tokio::io::{AsyncRead, AsyncReadExt};

use super::{Cookies, Error, FormData, FromFormData, Head, Params, Response, ToParameter};
use crate::prelude::ToConnectionRequest;
use crate::{
    config::get_config,
    controller::{Session, SessionId},
    model::Model,
    view::ToTemplateValue,
};

/// HTTP request.
#[derive(Debug, Clone)]
pub struct Request {
    head: Head,
    session: Session,
    inner: Arc<Inner>,
    params: Option<Arc<Params>>,
    received_at: OffsetDateTime,
    // Don't check for valid CSRF token.
    skip_csrf: bool,
    renew_session: bool,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            head: Head::default(),
            session: Session::default(),
            inner: Arc::new(Inner::default()),
            params: None,
            received_at: OffsetDateTime::now_utc(),
            skip_csrf: false,
            renew_session: false,
        }
    }
}

#[derive(Debug, Clone)]
struct Inner {
    body: Vec<u8>,
    cookies: Cookies,
    peer: SocketAddr,
}

impl Default for Inner {
    fn default() -> Inner {
        Inner {
            body: Vec::default(),
            cookies: Cookies::default(),
            peer: "127.0.0.1:8000".parse().unwrap(), // Just used for testing.
        }
    }
}

impl Request {
    /// Read the request in its entirety from a stream.
    ///
    /// #### Implementation note
    ///
    /// The request is fully received and loaded into memory before it's passed to a controller.
    /// It's safe to clone since the contents are behind an [`std::sync::Arc`].
    pub async fn read(peer: SocketAddr, mut stream: impl AsyncRead + Unpin) -> Result<Self, Error> {
        let head = Head::read(&mut stream).await?;
        let content_length = head.content_length().unwrap_or(0);

        // Handle requests which are too large.
        if content_length > get_config().general.max_request_size {
            // Throw away whatever we receive.
            let mut throw_away = vec![0u8; 4096];
            let mut content_length = content_length as i64;

            loop {
                let read = stream.read(&mut throw_away).await?;
                content_length -= read as i64;

                if content_length <= 0 || read == 0 {
                    break;
                }
            }

            return Err(Error::ContentTooLarge(head));
        }

        let mut body = vec![0u8; content_length];
        stream
            .read_exact(&mut body)
            .await
            .map_err(|_| Error::MalformedRequest("incorrect content length"))?;

        let cookies = head.cookies();

        let (session, renew_session) = match cookies.get_session()? {
            Some(session) => (session, false),
            None => (Session::anonymous(), true),
        };

        Ok(Request {
            head,
            params: None,
            session,
            inner: Arc::new(Inner {
                body,
                peer,
                cookies,
            }),
            received_at: OffsetDateTime::now_utc(),
            skip_csrf: false,
            renew_session,
        })
    }

    /// Get the request source IP address.
    ///
    /// This is the IP address of the TCP socket, and does
    /// not have to be the actual client's IP address.
    pub fn peer(&self) -> &SocketAddr {
        &self.inner.peer
    }

    /// Set params on the request.
    pub fn with_params(mut self, params: Arc<Params>) -> Self {
        self.params = Some(params);
        self
    }

    /// Return request head (headers, method, etc.).
    ///
    /// [`crate::http::Head`] is dereferenced from this struct,
    /// so all its public methods are available directly.
    pub fn head(&self) -> &Head {
        &self.head
    }

    /// Get mutable reference to the head.
    pub fn head_mut(&mut self) -> &mut Head {
        &mut self.head
    }

    /// Extract a parameter from the provided path.
    ///
    /// The parameter must be specified
    /// in the path provided to the router at controller registration. The only exception
    /// is the `id` parameter which is automatically configured on REST controllers.
    pub fn parameter<T: ToParameter>(&self, name: &str) -> Result<Option<T>, Error> {
        if let Some(ref params) = self.params {
            if let Some(parameter) = params.parameter(self.path().base(), name) {
                return Ok(Some(T::to_parameter(&parameter)?));
            }
        }

        Ok(None)
    }

    /// Retrieve the reequest body as bytes.
    ///
    /// It's the job of the caller to handle encoding, if any.
    pub fn body(&self) -> &[u8] {
        &self.inner.body
    }

    /// Request body parsed JSON value. If the body isn't JSON, an error is returned.
    pub fn json_raw(&self) -> Result<Value, serde_json::Error> {
        self.json()
    }

    /// Request body, parsed as a UTF-8 string.
    ///
    /// The encoding is assumed to be valid, and all incompatible characters are dropped.
    pub fn string(&self) -> String {
        String::from_utf8_lossy(self.body()).to_string()
    }

    /// Return data submitted via a form.
    ///
    /// If no data is submitted or the encoding is incorrect,
    /// an error is returned. Combined with the `?` operator inside a controller,
    /// this will automatically return `400 - Bad Request` to the client.
    pub fn form_data(&self) -> Result<FormData, Error> {
        FormData::from_request(self)
    }

    /// Return data submitted via a form, type checked
    /// with a Rust struct.
    ///
    /// This allows to check inputs of complex forms easily,
    /// or return a `400 - Bad Request` automatically if not (using the `?` operator).
    pub fn form<T: FromFormData>(&self) -> Result<T, Error> {
        T::from_form_data(&self.form_data()?)
    }

    /// Deserialize request body from JSON into a Rust struct. If deserialization fails,
    /// an error is returned.
    pub fn json<'a, T: Deserialize<'a>>(&'a self) -> Result<T, serde_json::Error> {
        let mut deserializer = Deserializer::from_slice(self.body());
        T::deserialize(&mut deserializer)
    }

    /// Return cookies set on the request. If no cookies are set,
    /// an empty [`crate::http::Cookies`] is returned.
    pub fn cookies(&self) -> &Cookies {
        &self.inner.cookies
    }

    /// Get the session set on the request, if any.
    ///
    /// All Rwf requests will have a session. If a browser doesn't save cookies (e.g. cURL doesn't),
    /// a new session will be generated for each request.
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Was the CSRF protection bypassed on this request?
    ///
    /// Used internally to skip CSRF middleware, but
    /// can also be used to check if the request data is safe to use.
    pub fn skip_csrf(&self) -> bool {
        self.skip_csrf
    }

    /// Return the timestamp of when the request was received by the server.
    pub fn received_at(&self) -> OffsetDateTime {
        self.received_at
    }

    /// Get the session identifier.
    ///
    /// This should uniquely identify a browser if it's a guest session,
    /// or a user if the user is logged in.
    pub fn session_id(&self) -> SessionId {
        self.session.session_id.clone()
    }

    /// Get the authenticated user's ID. Combined with the `?` operator,
    /// will return `401 - Unauthorized` if not logged in.
    pub fn user_id(&self) -> Result<i64, Error> {
        match self.session_id() {
            SessionId::Authenticated(id) => Ok(id),
            _ => Err(Error::Unauthorized),
        }
    }

    /// If a user is logged in, fetch the user's data from the database
    /// using the specified model.
    ///
    /// #### Example
    ///
    /// ```rust,ignore
    /// use rwf::prelude::*;
    ///
    /// #[derive(Clone, macros::Model)]
    /// struct User {
    ///     id: Option<i64>,
    ///     email: String,
    /// }
    ///
    /// let conn = Pool::connection().await?;
    /// let user = request.user::<User>(&mut conn).await?;
    /// ```
    pub async fn user<T: Model>(
        &self,
        conn: impl ToConnectionRequest<'_>,
    ) -> Result<Option<T>, Error> {
        match self.session_id() {
            SessionId::Authenticated(user_id) => Ok(Some(T::find(user_id).fetch(conn).await?)),

            _ => Ok(None),
        }
    }

    /// Same function as [`Request::user`], except if returns a [`Result`] instead of an [`Option`].
    /// If used with the `?` operator, returns `401 - Unauthorized` automatically.
    pub async fn user_required<T: Model>(
        &self,
        conn: impl ToConnectionRequest<'_>,
    ) -> Result<T, Error> {
        match self.user(conn).await? {
            Some(user) => Ok(user),
            None => Err(Error::Unauthorized),
        }
    }

    /// Set the session on the request. *For internal use only.*
    ///
    /// This is automatically done by the HTTP server,
    /// if the session is available.
    pub(crate) fn set_session(mut self, session: Session) -> Self {
        self.session = session;
        self.renew_session = true;
        self
    }

    /// Bypass CSRF protection. *For intenral use only.*
    ///
    /// Setting this on a response inside a controller does nothing since CSRF
    /// protection is invoked before the request reaches a controller.
    pub fn set_skip_csrf(mut self, skip_csrf: bool) -> Self {
        self.skip_csrf = skip_csrf;
        self
    }

    /// Did the client request a HTTP connection upgrade to WebSocket?
    pub fn upgrade_websocket(&self) -> bool {
        self.headers()
            .get("connection")
            .map(|v| v.to_lowercase().contains("upgrade"))
            == Some(true)
            && self.headers().get("upgrade").map(|v| v.to_lowercase())
                == Some(String::from("websocket"))
    }

    /// Log the user in. This creates a response with the session cookie set.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::prelude::*;
    /// # let request = Request::default();
    /// let response = request.login(1234);
    /// ```
    pub fn login(&self, user_id: i64) -> Response {
        let mut session = self.session.clone();
        session.session_id = SessionId::Authenticated(user_id);
        Response::new().set_session(session).html("")
    }

    /// Log the user in. Unlike [`Self::login`], this accepts any database model,
    /// and creates a response with the session cookie set.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::prelude::*;
    /// #[derive(Clone, macros::Model)]
    /// struct User {
    ///     id: Option<i64>,
    ///     email: String,
    /// }
    ///
    /// let user = User { id: Some(25), email: "test@test.com".into() };
    /// # let request = Request::default();
    /// let response = request.login_user(&user).unwrap();
    /// ```
    pub fn login_user(&self, user: &impl Model) -> Result<Response, Error> {
        use crate::model::Value;

        let get_user_id = |value: Value| -> Result<Response, Error> {
            match value {
                Value::Integer(user_id) => Ok(self.login(user_id)),
                Value::BigInt(user_id) => Ok(self.login(user_id)),
                Value::Int(user_id) => Ok(self.login(user_id as i64)),
                Value::SmallInt(user_id) => Ok(self.login(user_id as i64)),
                _ => Err(Error::UserIdNotAnInteger),
            }
        };

        match user.id() {
            Value::Optional(value) => match *value {
                Some(user_id) => get_user_id(user_id),
                None => Err(Error::UserIdIsNull),
            },
            value => get_user_id(value),
        }
    }

    /// Log the user out. This overwrites the session cookie with a guest session.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::prelude::*;
    /// # let request = Request::default();
    /// let response = request.logout();
    /// ```
    pub fn logout(&self) -> Response {
        Response::new().set_session(Session::anonymous()).html("")
    }

    pub(crate) fn renew_session(&self) -> bool {
        self.renew_session
    }
}

impl Deref for Request {
    type Target = Head;

    fn deref(&self) -> &Self::Target {
        &self.head
    }
}

impl ToTemplateValue for Request {
    fn to_template_value(&self) -> Result<crate::view::Value, crate::view::Error> {
        use crate::view::Value;

        let mut hash = HashMap::new();
        hash.insert("path".to_string(), self.path().base().to_template_value()?);
        hash.insert(
            "query".to_string(),
            self.path().query().to_string().to_template_value()?,
        );
        hash.insert("session".to_string(), self.session().to_template_value()?);
        Ok(Value::Hash(hash))
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

        let req = Request::read("127.0.0.1:1234".parse().unwrap(), &body[..]).await?;

        Ok(req)
    }

    pub fn dummy_ip() -> SocketAddr {
        "127.0.0.1:1234".parse().unwrap()
    }

    #[tokio::test]
    async fn test_json() {
        #[derive(Deserialize)]
        struct Hello {
            hello: String,
        }

        let request = dummy_request().await.unwrap();
        let json = request.json::<Hello>().expect("deserialize body");
        assert_eq!(json.hello, "world");
        assert_eq!(
            request.json_raw().unwrap(),
            serde_json::json!({
                "hello": "world",
            })
        );
    }

    #[tokio::test]
    async fn test_basic_req() {
        let normal = "GET /apples?hello=world HTTP/1.1\r\n".to_owned()
            + "Content-Length: 5\r\n\r\n"
            + "12345";
        let req = Request::read(dummy_ip(), normal.as_bytes()).await.unwrap();
        assert_eq!(req.body(), "12345".as_bytes());
        assert_eq!(req.content_length(), Some(5));
        assert_eq!(req.peer(), &dummy_ip());
        assert_eq!(req.upgrade_websocket(), false);
        assert_eq!(req.skip_csrf(), false);
        assert!(!req.session().authenticated());
        assert!(req.user_id().is_err());
        assert_eq!(req.body(), b"12345");
        assert_eq!(req.string(), "12345".to_string());
        assert!(req.form_data().is_err());
        assert_eq!(req.query().len(), 1);
        assert_eq!(req.path().base(), "/apples");

        let template_value = req.to_template_value().unwrap();
        use crate::view::Value;
        match template_value {
            Value::Hash(req) => {
                assert_eq!(req["query"], Value::String("hello=world".into()));
                assert_eq!(req["path"], Value::String("/apples".into()));
            }

            _ => panic!("request should be a hash"),
        };
    }

    #[tokio::test]
    async fn test_too_large() {
        // Test too large request.

        // We don't need to allocate them all,
        // they will be ignored.
        let b = vec![0u8; 12345];

        let mut too_large = "GET / HTTP/1.1\r\n".as_bytes().to_vec();
        too_large.extend("Content-Length: 123456789\r\n\r\n".as_bytes());
        too_large.extend(b);

        let req = Request::read(dummy_ip(), too_large.as_slice()).await;
        let err = req.expect_err("should err");
        let err = format!("{:?}", err);
        assert!(err.starts_with("ContentTooLarge"));
    }

    #[tokio::test]
    async fn test_login_logout() {
        let req = "GET / HTTP/1.1\r\nContent-Length: 0\r\n\r\n";
        let req = Request::read(dummy_ip(), req.as_bytes()).await.unwrap();
        let response = req.login(1234);
        assert!(response.session().is_some());
        assert!(response.session().as_ref().unwrap().authenticated());

        let response = req.logout();
        assert!(response.session().is_some());
        assert!(response.session().as_ref().unwrap().guest());
    }
}
