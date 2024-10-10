//! HTTP URL path, e.g. `/api/orders/1?sorted=true#header1`
//!
//! Paths are parsed for each incoming request and compared against
//! a global regex to find a route handler.
use super::{urldecode, Error};

use std::fmt::Debug;
use std::path::PathBuf;

pub mod with_regex;
pub use with_regex::{PathType, PathWithRegex};

pub mod to_parameter;
pub use to_parameter::ToParameter;

pub mod params;
pub use params::Params;

pub mod query;
pub use query::Query;

/// HTTP URL path.
#[derive(Clone, Debug)]
pub struct Path {
    query: Query,
    base: String,
}

impl Default for Path {
    fn default() -> Self {
        Path {
            query: Query::new(),
            base: "/".to_string(),
        }
    }
}

impl Path {
    pub fn from_parts(base: &str, query: &Query) -> Self {
        Self {
            base: base.to_string(),
            query: query.clone(),
        }
    }

    /// Path URL base.
    pub fn base(&self) -> &str {
        &self.base
    }

    /// Path length.
    pub fn len(&self) -> usize {
        self.base.len()
    }

    pub fn query(&self) -> &Query {
        &self.query
    }

    pub fn path(&self) -> &str {
        &self.base
    }

    pub fn parse(path: &str) -> Result<Path, Error> {
        // All paths must be absolute.
        let path = if path.starts_with("/") {
            path.to_string()
        } else {
            "/".to_string() + &path
        };

        // Parse the query.
        let parts = path.split("?").collect::<Vec<_>>();

        let (base, query) = match parts.len() {
            // Path has no query.
            1 => (path, Query::new()),

            // Path has a query.
            2 => {
                let mut query = Query::new();
                // Remove the anchor if any.
                let without_anchor = parts[1].split("#").next().expect("path anchor");
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

                (parts[0].to_owned(), query)
            }

            _ => return Err(Error::MalformedRequest("path has malformed query")),
        };

        Ok(Path { base, query })
    }

    pub fn to_std(&self) -> PathBuf {
        std::path::Path::new(&self.base).to_owned()
    }

    pub fn with_regex(self, path_type: PathType) -> Result<PathWithRegex, Error> {
        PathWithRegex::new(self, path_type)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_path() {
        let path = "/hello?foo=bar&hello%3Dworld";
        let path = Path::parse(path).unwrap();
        assert_eq!(path.path(), "/hello");
        assert_eq!(path.query().get("foo"), Some("bar".to_string()));
    }

    #[test]
    fn test_ordering() {
        assert!("asd" < "asdf");
    }

    #[test]
    fn test_regex() {
        let path = Path::parse("/api/orders/:id")
            .unwrap()
            .with_regex(PathType::Wildcard)
            .unwrap();
        let regex = path.regex();
        assert!(regex.find("/api/orders/1").is_some());
        assert!(regex.find("/api/orders").is_none());
        assert!(regex.find("/api/orders/hello/world").is_some());
    }
}
