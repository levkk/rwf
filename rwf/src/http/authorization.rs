//! Handles parsing the `Authorization` header.
use base64::prelude::*;

/// Authorization header.
#[derive(Debug, PartialEq)]
pub enum Authorization {
    /// HTTP Basic authentication.
    ///
    /// Basic auth uses a username and password. It's not very secure,
    /// but it's good enough to protect against random visitors.
    Basic { user: String, password: String },

    /// See [`crate::controller::auth::Token`].
    Token { token: String },

    /// Bearer authentication. Token validation is up to the caller.
    Bearer { token: String },
}

impl Authorization {
    /// Parse the `Authorization` header value.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::http::Authorization;
    /// let auth = Authorization::parse("Basic YWxpY2U6d29uZGVybGFuZA==");
    ///
    /// assert_eq!(auth, Some(Authorization::Basic {
    ///     user: "alice".into(),
    ///     password: "wonderland".into(),
    /// }));
    /// ```
    pub fn parse(header: &str) -> Option<Authorization> {
        let mut parts = header.split(" ");
        match parts.next() {
            Some(ident) => match ident {
                "Basic" => match parts.next() {
                    Some(auth) => Self::basic(auth),
                    None => None,
                },

                "Bearer" => match parts.next() {
                    Some(auth) => Some(Authorization::Bearer {
                        token: auth.to_owned(),
                    }),
                    None => None,
                },

                "Token" => match parts.next() {
                    Some(auth) => Some(Authorization::Token {
                        token: auth.to_owned(),
                    }),
                    None => None,
                },

                _ => None,
            },

            None => None,
        }
    }

    fn basic(value: &str) -> Option<Authorization> {
        if let Ok(decoded) = BASE64_STANDARD.decode(value.as_bytes()) {
            let decoded = String::from_utf8_lossy(&decoded);
            let mut parts = decoded.split(":");

            if let Some(user) = parts.next() {
                if let Some(password) = parts.next() {
                    return Some(Authorization::Basic {
                        user: user.to_owned(),
                        password: password.to_owned(),
                    });
                }
            }
        }

        None
    }
}
