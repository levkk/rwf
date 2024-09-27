//! Path with regex is used to:
//!
//! 1. Route requests to a controller
//! 2. Extract parameters from the URL
//!
//! Parameters are denoted by the column-name notation, e.g. `:param1`.

use super::{Error, Params, Path};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

/// Construct a regex for the specified path.
/// This allows the [`Router`] to find this path when HTTP requests are received.
#[derive(Debug, Clone)]
pub struct PathWithRegex {
    path: Path,
    params: Arc<Params>,
}

impl PathWithRegex {
    /// Create the path-specified regex.
    pub fn new(path: Path) -> Result<Self, Error> {
        let mut params = HashMap::new();
        let mut i = 1;
        let mut iter = path.base().split("/").peekable();
        let mut regex = Vec::new();
        while let Some(part) = iter.next() {
            let re = if part.starts_with(":") {
                params.insert(part[1..].to_owned(), i);
                i += 1;
                "([a-zA-Z0-9_-]+)"
            } else {
                part
            };
            regex.push(re);
        }
        let regex = "^".to_string() + &regex.join(r#"\/"#) + r#"(\/[a-zA-Z0-9_-]+)?"# + r#"\/?$"#;
        params.insert("id".to_string(), i);

        let regex = Regex::new(&regex)?;
        Ok(Self {
            path,
            params: Arc::new(Params::new(regex, params)),
        })
    }

    /// Get the params handler.
    pub fn params(&self) -> Arc<Params> {
        self.params.clone()
    }

    /// Get the regex used to route to this path.
    pub fn regex(&self) -> &Regex {
        self.params.regex()
    }
}

impl std::ops::Deref for PathWithRegex {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_paramters() {
        let path = Path::parse("/api/orders/:name/receipt").unwrap();
        let with_regex = PathWithRegex::new(path).unwrap();
        let params = with_regex.params();

        let url = "/api/orders/apple_bees/receipt/5";

        let name = params.parameter(url, "name");
        assert_eq!(name, Some("apple_bees"));

        let name = params.parameter(url, "id");
        assert_eq!(name, Some("5"));

        let url = "/api/orders/hello-world/receipt/";
        let name = params.parameter(url, "name");
        assert_eq!(name, Some("hello-world"));
    }
}
