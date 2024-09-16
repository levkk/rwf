use super::{urldecode, Error};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Clone, PartialEq, Debug)]
pub struct Path {
    query: HashMap<String, String>,
    base: String,
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
        let path = urldecode(path);
        let parts = path.split("?").collect::<Vec<_>>();
        match parts.len() {
            1 => Ok(Path {
                query: HashMap::new(),
                base: path.to_owned(),
            }),

            2 => {
                let mut query = HashMap::new();
                let query_parts = parts[1].split("&");
                for part in query_parts {
                    let key_value = part.split("=").collect::<Vec<_>>();
                    if key_value.len() != 2 {
                        continue;
                    }

                    let key = key_value.first().unwrap();
                    let value = key_value.last().unwrap();

                    query.insert(key.to_string(), value.to_string());
                }

                Ok(Path {
                    query,
                    base: parts[0].to_owned(),
                })
            }

            _ => Err(Error::MalformedRequest("path has malformed query")),
        }
    }
}

pub trait ToResource: Sync + Send {
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
}
