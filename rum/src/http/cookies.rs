use std::collections::HashMap;
use time::{Duration, OffsetDateTime};

use super::url::urldecode;
use super::Error;
use crate::config::get_config;
use crate::crypto::{decrypt, encrypt};

#[derive(Debug, Clone, Default)]
pub struct Cookies {
    cookies: HashMap<String, Cookie>,
}

impl Cookies {
    pub fn parse(value: &str) -> Cookies {
        let parts = value.split(";");
        let mut cookies = HashMap::new();

        for part in parts {
            if let Some(cookie) = Cookie::parse(part) {
                cookies.insert(cookie.name.to_string(), cookie);
            }
        }

        Cookies { cookies }
    }

    pub fn add_private(&mut self, cookie: impl ToCookie) -> Result<(), Error> {
        let mut cookie = cookie.to_cookie();
        cookie.value = encrypt(cookie.value.as_bytes())?;
        self.cookies.insert(cookie.name.clone(), cookie);

        Ok(())
    }

    pub fn get_private(&mut self, name: &str) -> Result<Option<Cookie>, Error> {
        if let Some(cookie) = self.cookies.get(name) {
            let mut cookie = cookie.clone();
            cookie.value = String::from_utf8_lossy(&decrypt(&cookie.value)?).to_string();
            Ok(Some(cookie))
        } else {
            Ok(None)
        }
    }

    pub fn add(&mut self, cookie: impl ToCookie) {
        let cookie = cookie.to_cookie();
        self.cookies.insert(cookie.name.clone(), cookie);
    }
}

impl std::fmt::Display for Cookies {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut result = Vec::new();
        for (_name, cookie) in &self.cookies {
            result.push(cookie.to_string());
        }
        write!(f, "{}", result.join(";"))
    }
}

pub trait ToCookie {
    fn to_cookie(self) -> Cookie;
}

impl ToCookie for (&str, &str) {
    fn to_cookie(self) -> Cookie {
        Cookie::new(self.0).with_value(self.1)
    }
}

impl ToCookie for (String, String) {
    fn to_cookie(self) -> Cookie {
        Cookie::new(self.0).with_value(self.1)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Cookie {
    name: String,
    value: String,
    expiration: Option<OffsetDateTime>,
    max_age: Option<Duration>,
    path: Option<String>,
    domain: Option<String>,
    http_only: bool,
    secure: bool,
}

impl Cookie {
    pub fn parse(value: &str) -> Option<Self> {
        let mut parts = value.split(";");
        let mut cookie = if let Some(cookie) = parts.next() {
            match Self::key_value(cookie) {
                (Some(key), Some(value)) => Cookie::new(&key).with_value(&urldecode(&value)),
                (Some(key), None) => Cookie::new(&key),
                _ => return None,
            }
        } else {
            return None;
        };

        for part in parts {
            match Self::key_value(part) {
                (Some(key), value) => match key.as_str() {
                    "Domain" => cookie.domain = value,
                    "HttpOnly" => cookie.http_only = true,
                    "Secure" => cookie.secure = true,
                    "Max-Age" => {
                        if let Some(value) = value {
                            match value.parse::<i64>() {
                                Ok(value) => cookie.max_age = Some(Duration::seconds(value)),
                                Err(_) => (),
                            }
                        }
                    }
                    _ => continue,
                },

                _ => continue,
            }
        }

        Some(cookie)
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

    pub fn new(name: impl ToString) -> Self {
        Cookie {
            name: name.to_string(),
            max_age: Some(get_config().cookie_max_age),
            ..Default::default()
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn with_value(mut self, value: impl ToString) -> Self {
        self.value = value.to_string();
        self
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
        }

        if let Some(ref domain) = self.domain {
            write!(f, "; Domain={}", domain)?;
        }

        Ok(())
    }
}
