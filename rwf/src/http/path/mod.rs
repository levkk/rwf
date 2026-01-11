//! HTTP URL path, e.g. `/api/orders/1?sorted=true#header1`
//!
//! Paths are parsed for each incoming request and compared against
//! a global regex to find a route handler.
use super::Error;

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
#[derive(Clone, Debug, crate::prelude::Deserialize, crate::prelude::Serialize)]
pub struct Path {
    query: Query,
    base: String,
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.base)?;
        if !self.query.is_empty() {
            write!(f, "?{}", self.query)?;
        }

        Ok(())
    }
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
    /// Construct Path from base URL and a parsed query string.
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

    /// Get the parsed query.
    pub fn query(&self) -> &Query {
        &self.query
    }

    /// Get the base path.
    pub fn path(&self) -> &str {
        &self.base
    }

    /// Parse the path from a string.
    ///
    /// # Example
    ///
    /// ```
    /// # use rwf::http::Path;
    /// let path = Path::parse("/users?id=5").unwrap();
    ///
    /// assert_eq!(path.path(), "/users");
    /// assert_eq!(path.query().get::<i64>("id").unwrap(), 5);
    /// ```
    pub fn parse(path: &str) -> Result<Path, Error> {
        // All paths must be absolute.
        let path = if path.starts_with("/") {
            path.to_string()
        } else {
            "/".to_string() + path
        };

        // Parse the query.
        let parts = path.split("?").collect::<Vec<_>>();

        let (base, query) = match parts.len() {
            // Path has no query.
            1 => (path, Query::new()),

            // Path has a query.
            2 => (parts[0].to_owned(), Query::parse(parts[1])),

            _ => return Err(Error::MalformedRequest("path has malformed query")),
        };

        Ok(Path { base, query })
    }

    /// Convert path to a path that can be read from the file system.
    pub fn to_std(&self) -> PathBuf {
        std::path::Path::new(&self.base).to_owned()
    }

    /// Constuct a matching regex for this path. Used for matching of
    /// incoming requests to a controller.
    pub fn with_regex(self, path_type: PathType) -> Result<PathWithRegex, Error> {
        PathWithRegex::new(self, path_type)
    }

    /// Remove the base path from this path, e.g.
    /// `/engine/users/1` with the base `/engine` removed is `/users/1`.
    pub fn pop_base(&self, base: &Path) -> Self {
        let mut new_base = String::new();
        let mut iter = base.base().chars();
        for c in self.base.chars() {
            if let Some(c2) = iter.next() {
                if c2 == c {
                    continue;
                }
            }
            new_base.push(c);
        }
        Self::from_parts(&new_base, self.query())
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
        assert_eq!(path.query().get("hello=world"), Some("".to_string()));
        assert_eq!(path.to_string(), "/hello?foo=bar&hello%3Dworld=");
        assert_eq!(path.query().to_string(), "foo=bar&hello%3Dworld=");

        let path = "/blog";
        let path = Path::parse(path).unwrap();
        assert_eq!(path.to_string(), "/blog");
        assert!(path.query().is_empty());
        assert_eq!(path.query().to_string(), "");
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

    #[test]
    fn test_pop_base() {
        let path = Path::parse("/engine/users/1/engine").unwrap();
        let engine = Path::parse("/engine").unwrap();
        let path = path.pop_base(&engine);
        assert_eq!(path.base(), "/users/1/engine");
    }
}
