//! Handle parsing the `Authorization` header.
use base64::prelude::*;

#[derive(Debug, PartialEq)]
pub enum Authorization {
    /// HTTP Basic authentication
    Basic { user: String, password: String },

    /// See [`controller::auth::Token`].
    Token { token: String },

    /// Bearer authentication. Token validation is up to the caller.
    Bearer { token: String },
}

impl Authorization {
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
