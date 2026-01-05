//! HTTP cookies.
//!
//! This module handles decoding the `Cookie` header,
//! and generating `Set-Cookie` headers.
use std::collections::HashMap;
use time::{Duration, OffsetDateTime};

use super::url::urldecode;
use super::Error;
use crate::config::get_config;
use crate::controller::Session;
use crate::crypto::{decrypt, encrypt};

/// Cookies storage.
///
/// Supports both plain text and encrypted (private) cookies.
#[derive(Debug, Clone, Default, crate::prelude::Serialize, crate::prelude::Deserialize)]
pub struct Cookies {
    cookies: HashMap<String, Cookie>,
}

impl Cookies {
    /// Create new empty cookies storage.
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }

    /// Parse cookies from the `Cookie` header.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::http::Cookies;
    /// let cookies = Cookies::parse("rwf_aid=1234; rwf_session=foo");
    /// assert_eq!(
    ///     cookies
    ///         .get("rwf_aid")
    ///         .unwrap()
    ///         .value(),
    ///     "1234"
    /// );
    /// ```
    pub fn parse(value: &str) -> Cookies {
        let parts = value.split(";");
        let mut cookies = HashMap::new();

        for part in parts {
            if let Some(cookie) = Cookie::parse(part.trim()) {
                cookies.insert(cookie.name.to_string(), cookie);
            }
        }

        Cookies { cookies }
    }

    /// Add an encrypted cookie.
    ///
    /// If this is set on the [`crate::http::Response`], this cookie will be sent
    /// to the client.
    pub fn add_private(&mut self, cookie: impl ToCookie) -> Result<(), Error> {
        let mut cookie = cookie.to_cookie();
        cookie.value = encrypt(cookie.value.as_bytes())?;
        self.cookies.insert(cookie.name.clone(), cookie);

        Ok(())
    }

    /// Get an encrypted cookie received from the client. The value is decrypted automatically.
    ///
    /// If the decryption fails, `None` is returned. This indicates the encrypted cookie has been modified,
    /// or has been encrypted with a different secret key.
    ///
    /// If the cookie isn't valid UTF-8, like the HTTP specification requires, an error is returned.
    pub fn get_private(&self, name: &str) -> Result<Option<Cookie>, Error> {
        if let Some(cookie) = self.cookies.get(name) {
            let mut cookie = cookie.clone();
            cookie.value = String::from_utf8(match decrypt(&cookie.value) {
                Ok(value) => value,
                Err(_) => return Ok(None),
            })?;
            Ok(Some(cookie))
        } else {
            Ok(None)
        }
    }

    /// Add a cookie.
    ///
    /// If this is done to the response, the cookie will be sent it to the client,
    /// using the `Set-Cookie` header.
    pub fn add(&mut self, cookie: impl ToCookie) {
        let cookie = cookie.to_cookie();
        self.cookies.insert(cookie.name.clone(), cookie);
    }

    /// Get a cookie sent by the client.
    pub fn get(&self, name: &str) -> Option<&Cookie> {
        self.cookies.get(name)
    }

    /// Get the session cookie, if one is set. If no session is set,
    /// `None` is returned. While all requests should have a session, there is
    /// no guarantee the browser respects cookie settings we send over (e.g. cURL won't).
    ///
    /// If the session is not valid UTF-8, an error is returned.
    pub fn get_session(&self) -> Result<Option<Session>, Error> {
        let cookie = self.get_private("rwf_session")?;

        if let Some(cookie) = cookie {
            Ok(serde_json::from_str(cookie.value())?)
        } else {
            Ok(None)
        }
    }

    /// Set a sessionn cookie and send it to the client. The cookie expires
    /// when the session does.
    pub fn add_session(&mut self, session: &Session) -> Result<(), Error> {
        let value = serde_json::to_string(session)?;
        self.add_private(
            CookieBuilder::new()
                .name("rwf_session")
                .value(value)
                .expiration(OffsetDateTime::from_unix_timestamp(session.expiration)?)
                .build(),
        )
    }

    /// Convert cookies to `Set-Cookie` headers which will be sent to the client.
    pub fn to_headers(&self) -> Vec<u8> {
        let mut headers = vec![];
        for (_, cookie) in &self.cookies {
            headers.extend_from_slice(format!("set-cookie: {}\r\n", cookie).as_bytes());
        }
        headers
    }
}

impl std::fmt::Display for Cookies {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut result = Vec::new();
        for (_name, cookie) in &self.cookies {
            result.push(cookie.to_string());
        }
        write!(f, "{}", result.join("; "))
    }
}

/// Convert a value to a cookie.
///
/// This is syntax sugar to help create cookies more easily. Most use cases would
/// want to use the [`CookieBuilder`] instead.
pub trait ToCookie {
    fn to_cookie(self) -> Cookie;
}

impl ToCookie for (&str, &str) {
    fn to_cookie(self) -> Cookie {
        let builder = CookieBuilder::new();
        builder.name(self.0).value(self.1).build()
    }
}

impl ToCookie for (String, String) {
    fn to_cookie(self) -> Cookie {
        let builder = CookieBuilder::new();
        builder.name(self.0).value(self.1).build()
    }
}

impl ToCookie for Cookie {
    fn to_cookie(self) -> Cookie {
        self
    }
}

/// A browser cookie.
#[derive(Debug, Clone, Default, crate::prelude::Serialize, crate::prelude::Deserialize)]
pub struct Cookie {
    name: String,
    value: String,
    expiration: Option<OffsetDateTime>,
    max_age: Option<Duration>,
    path: Option<String>,
    domain: Option<String>,
    http_only: bool,
    secure: bool,
    same_site: Option<String>,
}

impl Cookie {
    /// Parse a single cookie from the `Cookie` header.
    fn parse(value: &str) -> Option<Self> {
        let mut parts = value.split(";");
        let mut builder = CookieBuilder::new();
        let _cookie = if let Some(cookie) = parts.next() {
            match Self::key_value(cookie) {
                (Some(key), Some(value)) => builder = builder.name(&key).value(urldecode(&value)),
                (Some(key), None) => builder = builder.name(&key),
                _ => return None,
            }
        } else {
            return None;
        };

        for part in parts {
            match Self::key_value(part) {
                (Some(key), value) => match key.as_str().trim() {
                    "Domain" => {
                        if let Some(value) = value {
                            builder = builder.domain(value);
                        }
                    }
                    "HttpOnly" => {
                        builder = builder.http_only();
                    }
                    "Secure" => {
                        builder = builder.secure();
                    }
                    "Max-Age" => {
                        if let Some(value) = value {
                            match value.parse::<i64>() {
                                Ok(value) => {
                                    builder = builder.max_age(Duration::seconds(value));
                                }
                                Err(_) => continue,
                            }
                        }
                    }
                    _ => continue,
                },

                _ => continue,
            };
        }

        Some(builder.build())
    }

    fn key_value(s: &str) -> (Option<String>, Option<String>) {
        let mut parts = s.split("=");
        if let Some(key) = parts.next() {
            if let Some(value) = parts.next() {
                (Some(key.to_owned()), Some(value.to_owned()))
            } else {
                (Some(key.to_owned()), None)
            }
        } else {
            (None, None)
        }
    }

    /// Create new cookie with the name.
    fn new(name: impl ToString) -> Self {
        Cookie {
            name: name.to_string(),
            max_age: Some(get_config().general.cookie_max_age()),
            ..Default::default()
        }
    }

    /// Get cookie value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Get cookie name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Check if the cookie is secure.
    fn secure(&self) -> bool {
        self.secure
    }

    /// Check if the cookie is HTTP-only.
    fn http_only(&self) -> bool {
        self.http_only
    }

    /// Get the cookie's `MaxAge` attribute if any is set.
    fn max_age(&self) -> Option<Duration> {
        self.max_age
    }
}

impl std::fmt::Display for Cookie {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}={}", self.name, self.value)?;

        if let Some(ref max_age) = self.max_age {
            write!(f, "; Max-Age: {}", max_age.whole_seconds())?;
        }

        if self.secure {
            write!(f, "; Secure")?;
        }

        if self.http_only {
            write!(f, "; HttpOnly")?;
        }

        if let Some(ref path) = self.path {
            write!(f, "; Path={}", path)?;
        } else {
            write!(f, "; Path=/")?;
        }

        if let Some(ref domain) = self.domain {
            write!(f, "; Domain={}", domain)?;
        }

        if let Some(ref same_site) = self.same_site {
            write!(f, "; SameSite={}", same_site)?;
        } else {
            write!(f, "; SameSite=Lax")?;
        }

        if let Some(ref expiration) = self.expiration {
            write!(
                f,
                "; Expires={}",
                expiration
                    .format(&time::format_description::well_known::Rfc2822)
                    .unwrap()
            )?;
        }

        Ok(())
    }
}

/// Cookie builder which helps with creating cookies with multiple attributes.
///
/// # Example
///
/// ```
/// # use rwf::http::CookieBuilder;
/// use time::Duration;
///
/// let cookie = CookieBuilder::new()
///     .name("rwf_aid")
///     .value("1234")
///     .max_age(Duration::days(4))
///     .secure()
///     .http_only()
///     .build();
/// ```
///
/// The resulting cookie can be set on a response, which will send it to the client.
#[derive(Clone, Debug)]
pub struct CookieBuilder {
    cookie: Cookie,
}

impl CookieBuilder {
    /// Create new cookie builder.
    pub fn new() -> Self {
        Self {
            cookie: Cookie::default(),
        }
    }

    /// Set cookie name.
    pub fn name(mut self, name: impl ToString) -> Self {
        self.cookie.name = name.to_string();
        self
    }

    /// Set cookie value. The value is stored in plain text.
    pub fn value(mut self, value: impl ToString) -> Self {
        self.cookie.value = value.to_string();
        self
    }

    /// Set cookie `Expiration` attribute.
    pub fn expiration(mut self, expiration: OffsetDateTime) -> Self {
        self.cookie.expiration = Some(expiration);
        self
    }

    /// Set cookie `MaxAge` attribute.
    pub fn max_age(mut self, max_age: Duration) -> Self {
        self.cookie.max_age = Some(max_age);
        self
    }

    /// Set cookie `Path` attribute.
    pub fn path(mut self, path: impl ToString) -> Self {
        self.cookie.path = Some(path.to_string());
        self
    }

    /// Set cookie `Domain` attribute.
    pub fn domain(mut self, domain: impl ToString) -> Self {
        self.cookie.domain = Some(domain.to_string());
        self
    }

    /// Set the cookie to be only sent via plain HTTP requests (no AJAX).
    /// This is the `HttpOnly` attribute.
    pub fn http_only(mut self) -> Self {
        self.cookie.http_only = true;
        self
    }

    /// Make sure the cookie is sent only via HTTPS connections.
    /// This is the `Secure` attribute.
    pub fn secure(mut self) -> Self {
        self.cookie.secure = true;
        self
    }

    /// Set cookie `SameSite` attribute to `Lax`.
    ///
    /// This setting is desirable if you want
    /// the cookie set on redirects from external sites.
    pub fn lax(mut self) -> Self {
        self.cookie.same_site = Some("Lax".to_string());
        self
    }

    /// Set cookie `SameSite` attribute to `Strict`.
    ///
    /// This cookie won't be set on redirects from external links, breaking
    /// authentication.
    pub fn strict(mut self) -> Self {
        self.cookie.same_site = Some("Strict".to_string());
        self
    }

    /// Build the cookie.
    ///
    /// This consumes the builder.
    pub fn build(self) -> Cookie {
        self.cookie
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parsing_cookies() {
        let value = "name=some_value; Max-Age=55; Secure";
        let cookie = Cookie::parse(value).expect("cookie parse");
        assert_eq!(cookie.name(), "name");
        assert_eq!(cookie.value(), "some_value");
        assert!(cookie.secure());
        assert_eq!(cookie.max_age(), Some(Duration::seconds(55)));

        let value = "random=hello_world";
        let cookie = Cookie::parse(value).expect("cookie parse");
        assert_eq!(cookie.name(), "random");
        assert_eq!(cookie.value(), "hello_world");
    }

    #[test]
    fn test_creating_cookies() {
        let mut cookies = Cookies::new();
        cookies.add(("hello", "world"));
        cookies
            .add_private(("session", "super_secret_key"))
            .expect("private");
        let s = cookies.to_string();

        let cookies = Cookies::parse(&s);
        assert!(cookies.get("hello").is_some());
        assert_eq!(cookies.get("hello").unwrap().value(), "world");
        assert_eq!(
            cookies
                .get_private("session")
                .expect("decrypt")
                .expect("session cookie")
                .value(),
            "super_secret_key"
        );
    }
}
