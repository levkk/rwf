use super::{urldecode, Error};
use std::cmp::{Ordering, PartialOrd};
use std::collections::HashMap;
use std::fmt::Debug;

use regex::Regex;

#[derive(Clone, Debug)]
pub struct Path {
    query: HashMap<String, String>,
    base: String,
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.base.eq(&other.base)
    }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.base.partial_cmp(&other.base)
    }
}

impl Eq for Path {}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> Ordering {
        self.base.cmp(&other.base)
    }
}

impl Default for Path {
    fn default() -> Self {
        Path {
            query: HashMap::new(),
            base: "/".to_string(),
        }
    }
}

impl Path {
    pub fn base(&self) -> &str {
        &self.base
    }

    pub fn len(&self) -> usize {
        self.base.len()
    }

    pub fn matches(&self, path: &Path) -> bool {
        let it_matches = self.base.starts_with(&path.base);
        it_matches
    }

    pub fn is_root(&self) -> bool {
        self.base.ends_with("/")
    }

    pub fn resource<T: ToResource>(&self) -> Option<Result<T, Error>> {
        if self.is_root() {
            None
        } else {
            let reverse_offset = self.base.chars().rev().position(|c| c == '/').unwrap_or(0);
            let last_slash_offset = self.base.len() - reverse_offset;

            if last_slash_offset < self.base.len() {
                Some(T::to_resource(&self.base[last_slash_offset..]))
            } else {
                None
            }
        }
    }

    pub fn query(&self) -> &HashMap<String, String> {
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
            1 => (path, HashMap::new()),

            // Path has a query.
            2 => {
                let mut query = HashMap::new();
                // Remove the anchor if any.
                let without_anchor = parts[1].split("#").next().expect("path anchor");
                let query_parts = without_anchor.split("&");
                for part in query_parts {
                    let key_value = part.split("=").collect::<Vec<_>>();
                    if key_value.len() != 2 {
                        continue;
                    }

                    // Decode any URL-encoded values back into UTF-8.
                    let key = urldecode(&key_value.first().expect("path query key"));
                    let value = urldecode(&key_value.last().expect("path query value"));

                    query.insert(key, value);
                }

                (parts[0].to_owned(), query)
            }

            _ => return Err(Error::MalformedRequest("path has malformed query")),
        };

        Ok(Path { base, query })
    }

    pub fn with_regex(self) -> Result<PathWithRegex, Error> {
        PathWithRegex::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct PathWithRegex {
    regex: Regex,
    path: Path,
}

impl PathWithRegex {
    pub fn new(path: Path) -> Result<Self, Error> {
        let regex = path
            .base()
            .split("/")
            .map(|p| {
                if p.starts_with(":") {
                    "([a-zA-Z0-9]+)"
                } else {
                    p
                }
            })
            .collect::<Vec<_>>();
        let mut regex = "^".to_string() + &regex.join("/");
        regex.push_str("(.*)");

        let regex = Regex::new(&regex)?;
        Ok(Self { regex, path })
    }

    pub fn regex_pattern(&self) -> &str {
        self.regex.as_str()
    }

    pub fn regex(&self) -> &Regex {
        &self.regex
    }
}

impl std::ops::Deref for PathWithRegex {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

pub trait ToResource: Sync + Send + Debug {
    fn to_resource(s: &str) -> Result<Self, Error>
    where
        Self: Sized;
}

impl ToResource for i64 {
    fn to_resource(s: &str) -> Result<i64, Error> {
        match s.parse() {
            Ok(id) => Ok(id),
            Err(_) => Err(Error::MalformedRequest("i64")),
        }
    }
}

impl ToResource for String {
    fn to_resource(s: &str) -> Result<String, Error> {
        Ok(s.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_path() {
        let path = "/hello?foo=bar&hello%3Dworld";
        let path = Path::parse(path).unwrap();
        assert_eq!(path.path(), "/hello");
        assert_eq!(path.query().get("foo"), Some(&"bar".to_string()));
    }

    #[test]
    fn test_path_resource() {
        let path = "/hello/world?foo=bar";
        let path = Path::parse(path).unwrap();
        let resource = path.resource::<String>().unwrap().unwrap();
        assert_eq!(resource, "world".to_string());

        let path = "/hello/?foo=bar&hello=world";
        let path = Path::parse(path).unwrap();
        assert!(path.resource::<String>().is_none());

        let path = "/?foo=bar";
        let path = Path::parse(path).unwrap();
        assert!(path.resource::<String>().is_none());

        let path = "/hello/1";
        let path = Path::parse(path).unwrap();
        assert_eq!(path.resource::<i64>().unwrap().unwrap(), 1);
    }

    #[test]
    fn test_ordering() {
        assert!("asd" < "asdf");
    }

    #[test]
    fn test_regex() {
        let path = Path::parse("/api/orders/:id")
            .unwrap()
            .with_regex()
            .unwrap();
        let regex = Regex::new(path.regex_pattern()).expect("to be a valid regex");
        assert!(regex.find("/api/orders/1").is_some());
        assert!(regex.find("/api/orders").is_none());
        assert!(regex.find("/api/orders/hello/world").is_some());
    }
}
