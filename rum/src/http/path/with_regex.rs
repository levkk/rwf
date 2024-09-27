use super::{Error, Params, Path};
use regex::Regex;
use std::sync::Arc;

/// Construct a regex for the specified path.
/// This allows the [`Router`] to find this path when HTTP requests are received.
#[derive(Debug, Clone)]
pub struct PathWithRegex {
    path: Path,
    params: Arc<Params>,
}

impl PathWithRegex {
    /// Create the path-specifid regex.
    pub fn new(path: Path) -> Result<Self, Error> {
        let mut params = vec![];
        let mut i = 1;
        let regex = path
            .base()
            .split("/")
            .map(|p| {
                if p.starts_with(":") {
                    params.push(i);
                    i += 1;
                    "([a-zA-Z0-9]+)"
                } else {
                    p
                }
            })
            .collect::<Vec<_>>();
        let mut regex = "^".to_string() + &regex.join("/");
        params.push(i);
        regex.push_str("/?([a-zA-Z0-9]+)?(.*)");

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

    #[test]
    fn test_paramter() {
        // let path = Path::parse("/api/orders/:id")
        //     .unwrap()
        //     .with_regex()
        //     .unwrap();
        // let req = Path::parse("/api/orders/123").unwrap();
        // let param = path.parameter::<i64>(&req, 0).expect("to have a parameter");
        // assert_eq!(param, 123);
    }
}
