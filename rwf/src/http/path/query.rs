use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use crate::http::urldecode;

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
            Some(value) => match value.parse::<T>() {
                Ok(value) => Some(value),
                Err(_) => None,
            },

            None => None,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.query).unwrap_or(serde_json::Value::default())
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
