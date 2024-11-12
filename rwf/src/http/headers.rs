//! HTTP request headers.
use std::collections::{hash_map::Iter, HashMap};

/// HTTP headers.
#[derive(Clone, Debug, Default)]
pub struct Headers {
    headers: HashMap<String, String>,
}

impl Headers {
    /// Create new empty headers.
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    /// Insert a header name and value.
    ///
    /// The name will be converted to lowercase.
    pub fn insert(&mut self, name: impl ToString, value: impl ToString) {
        self.headers
            .insert(name.to_string().to_lowercase(), value.to_string());
    }

    /// Get a header value by name. Case insensitive.
    pub fn get(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }

    /// Remove a header by name. Case insensitive.
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.headers.remove(&name.to_lowercase())
    }

    /// Remove all headers.
    pub fn clear(&mut self) {
        self.headers.clear();
    }

    /// Convert headers into a [`HashMap`] keyed by header name.
    pub fn into_raw(self) -> HashMap<String, String> {
        self.headers
    }

    /// Get a borrowing interator to the headers.
    pub fn iter(&self) -> Iter<String, String> {
        self.headers.iter()
    }

    /// Convert headers to bytes (UTF-8).
    ///
    /// Used to send headers over the wire to the client as part of a response.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (name, value) in &self.headers {
            bytes.extend_from_slice(name.as_bytes());
            bytes.extend_from_slice(b": ");
            bytes.extend_from_slice(value.as_bytes());
            bytes.extend_from_slice(b"\r\n");
        }
        bytes
    }
}

impl From<HashMap<String, String>> for Headers {
    fn from(headers: HashMap<String, String>) -> Self {
        Self { headers }
    }
}
