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
/// This allows the [`crate::http::Router`] to find this path when HTTP requests are received.
#[derive(Debug, Clone)]
pub struct PathWithRegex {
    path: Path,
    params: Arc<Params>,
    path_type: PathType,
}

/// Kind of path routing we are using for a controller.
#[derive(PartialEq, Debug, Clone)]
pub enum PathType {
    /// Will match all 6 REST paths.
    Rest,
    /// Will match all child paths and self.
    Wildcard,
    /// Will only match the specific path.
    Route,
}

impl PathWithRegex {
    /// Create the path-specified regex.
    pub(crate) fn new(path: Path, path_type: PathType) -> Result<Self, Error> {
        let mut params = HashMap::new();
        // Parameter regex groups start at 1 since the first group
        // is the path base URL.
        let mut i = 1;
        let iter = path.base().split("/");
        let mut regex = Vec::new();
        for part in iter {
            let re = if part.starts_with(":") {
                // Parameter name and group number.
                params.insert(part[1..].to_owned(), i);
                i += 1;
                "([a-zA-Z0-9_-]+)"
            } else {
                // Match the URL part as-is.
                part
            };
            regex.push(re);
        }
        let regex =
            // Start of the URL
            "^".to_string() +
            // URL parts joined by '/'
            &regex.join(r#"\/"#) +

            match path_type {
                PathType::Rest => {
                    // The :id parameter is optional
                    r#"(\/[a-zA-Z0-9_-]+)?"#
                }

                PathType::Route => "",
                PathType::Wildcard => ".*",
            }

            +

            // Last slash is optional
            if path.base().ends_with("/") { "$" } else { r#"\/?$"# };

        // :id parameter
        if path_type == PathType::Rest {
            params.insert("id".to_string(), i);
        }

        let regex = Regex::new(&regex)?;

        Ok(Self {
            path,
            params: Arc::new(Params::new(regex, params)),
            path_type,
        })
    }

    pub(crate) fn route(path: Path) -> Result<Self, Error> {
        Self::new(path, PathType::Route)
    }

    pub(crate) fn rest(path: Path) -> Result<Self, Error> {
        Self::new(path, PathType::Rest)
    }

    pub(crate) fn wildcard(path: Path) -> Result<Self, Error> {
        Self::new(path, PathType::Wildcard)
    }

    /// Get the params handler.
    pub fn params(&self) -> Arc<Params> {
        self.params.clone()
    }

    /// Get the regex used to route to this path.
    pub fn regex(&self) -> &Regex {
        self.params.regex()
    }

    /// Get the path type.
    pub fn path_type(&self) -> &PathType {
        &self.path_type
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
        let with_regex = PathWithRegex::rest(path).unwrap();
        assert_eq!(
            r#"^\/api\/orders\/([a-zA-Z0-9_-]+)\/receipt(\/[a-zA-Z0-9_-]+)?\/?$"#,
            with_regex.regex().as_str()
        );
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
