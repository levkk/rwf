use super::{Error, Path, ToParameter};
use regex::Regex;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PathWithRegex {
    regex: Arc<Regex>,
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

        let regex = Arc::new(Regex::new(&regex)?);
        Ok(Self { regex, path })
    }

    /// Extract a parameter from the provided path.
    pub fn parameter<T: ToParameter>(&self, path: &Path, index: usize) -> Option<T> {
        let captures = self.regex.captures(path.base());

        if let Some(captures) = captures {
            if let Some(capture) = captures.get(index) {
                if let Ok(param) = T::to_parameter(capture.as_str()) {
                    return Some(param);
                }
            }
        }

        None
    }

    pub fn regex_pattern(&self) -> &str {
        self.regex.as_str()
    }

    pub fn regex(&self) -> Arc<Regex> {
        self.regex.clone()
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
    fn test_paramter() {
        let path = Path::parse("/api/orders/:id")
            .unwrap()
            .with_regex()
            .unwrap();
        let req = Path::parse("/api/orders/123").unwrap();
        let param = path.parameter::<i64>(&req, 0).expect("to have a parameter");
        assert_eq!(param, 123);
    }
}
