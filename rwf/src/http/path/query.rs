use std::collections::{hash_map::IntoIter, HashMap};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use crate::http::urldecode;
use crate::http::Error;

#[derive(Debug, Clone)]
pub struct Query {
    query: HashMap<String, String>,
}

impl Query {
    pub fn new() -> Self {
        Self {
            query: HashMap::new(),
        }
    }

    pub fn parse(data: &str) -> Self {
        let mut query = Self::new();

        // Remove the anchor if any.
        let without_anchor = data.split("#").next().expect("path anchor");
        let query_parts = without_anchor.split("&");
        for part in query_parts {
            let key_value = part.split("=").collect::<Vec<_>>();

            if key_value.len() > 2 {
                continue;
            }

            // Decode any URL-encoded values back into UTF-8.
            let key = urldecode(&key_value.first().expect("path query key"));
            let value = urldecode(&key_value.last().unwrap_or(&"")); // ?key=&value=two

            query.insert(key, value);
        }

        query
    }

    pub fn get<T: FromStr>(&self, name: &str) -> Option<T> {
        match self.query.get(name) {
            Some(value) => match urldecode(value).parse::<T>() {
                Ok(value) => Some(value),
                Err(_) => None,
            },

            None => None,
        }
    }

    /// Get a parameter, returning HTTP 400 if it's not set.
    pub fn get_required<T: FromStr>(&self, name: &str) -> Result<T, Error> {
        match self.get(name) {
            Some(value) => Ok(value),
            None => Err(Error::MissingParameter),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.query).unwrap_or(serde_json::Value::default())
    }

    /// An owning iterator over the query.
    pub fn into_iter(self) -> IntoIter<String, String> {
        self.query.into_iter()
    }
}

impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut params = vec![];
        for (key, value) in &self.query {
            params.push(format!("{}={}", key, value));
        }

        write!(f, "?{}", params.join("&"))
    }
}

impl Deref for Query {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.query
    }
}

impl DerefMut for Query {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.query
    }
}
