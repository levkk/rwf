use super::Error;
use std::collections::HashMap;

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
    pub fn query(&self) -> &HashMap<String, String> {
        &self.query
    }
    pub fn path(&self) -> &str {
        &self.base
    }
    pub fn parse(path: &str) -> Result<Path, Error> {
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

            _ => Err(Error::MalformedRequest("malformed path")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_path() {
        let path = "/hello?foo=bar&hello=world";
        let path = Path::parse(path).unwrap();
        assert_eq!(path.path(), "/hello");
        assert_eq!(path.query().get("foo"), Some(&"bar".to_string()));
    }
}
