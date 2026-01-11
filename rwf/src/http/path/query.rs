//! Handles parsing the URL query.
use std::collections::btree_map::{BTreeMap, IntoIter};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use crate::http::Error;
use crate::http::{urldecode, urlencode};

/// GET request query.
///
/// # Example
///
/// ```text
/// page=5&page_size=25
/// ```
#[derive(Debug, Clone, crate::prelude::Deserialize, crate::prelude::Serialize)]
pub struct Query {
    query: BTreeMap<String, String>,
}

impl Query {
    /// Create new empty query.
    pub fn new() -> Self {
        Self {
            query: BTreeMap::new(),
        }
    }

    /// Parse query from a GET request.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::http::Query;
    /// let query = Query::parse("page=5&page_size=25");
    /// ```
    pub fn parse(data: &str) -> Self {
        let mut query = Self::new();

        // Remove the anchor if any.
        let without_anchor = data.split("#").next().expect("path anchor");
        let query_parts = without_anchor.split("&");
        for part in query_parts {
            let mut key_value = part.split("=").collect::<Vec<_>>().into_iter();

            if key_value.len() > 2 {
                continue;
            }

            // Decode any URL-encoded values back into UTF-8.
            let key = urldecode(key_value.next().expect("path query key"));
            let value = urldecode(key_value.next().unwrap_or("")); // ?key=&value=two

            query.insert(key, value);
        }

        query
    }

    /// Get a query parameter value. The parameter is converted
    /// to a Rust type. If the conversion fails, `None` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::http::Query;
    /// let query = Query::parse("page=5&page_size=25");
    /// assert_eq!(
    ///     query.get::<i64>("page"),
    ///     Some(5)
    /// );
    /// ```
    pub fn get<T: FromStr>(&self, name: &str) -> Option<T> {
        match self.query.get(name) {
            Some(value) => urldecode(value).parse::<T>().ok(),

            None => None,
        }
    }

    /// Get a query parameter value. If it's not set, return an error.
    /// When used with the `?` operator, the controller will automatically
    /// return `400 - Bad Request`.
    pub fn get_required<T: FromStr>(&self, name: &str) -> Result<T, Error> {
        match self.get(name) {
            Some(value) => Ok(value),
            None => Err(Error::MissingParameter),
        }
    }

    /// Convert the query to JSON representation.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::http::Query;
    /// assert_eq!(
    ///     Query::parse("page=25&page_size=5").to_json(),
    ///     serde_json::json!({
    ///         "page": "25",
    ///         "page_size": "5",
    ///     })
    /// )
    /// ```
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.query).unwrap_or_default()
    }

    /// An owning iterator over the query.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::http::Query;
    /// let query = Query::parse("page=25&page_size=5");
    ///
    /// for (key, value) in query.into_iter() {
    ///     // ...
    /// }
    /// ```
    pub fn into_iter(self) -> IntoIter<String, String> {
        self.query.into_iter()
    }
}

impl std::fmt::Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut params = vec![];
        for (key, value) in &self.query {
            params.push(format!("{}={}", urlencode(key), urlencode(value)));
        }

        write!(f, "{}", params.join("&"))
    }
}

impl Deref for Query {
    type Target = BTreeMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.query
    }
}

impl DerefMut for Query {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.query
    }
}
