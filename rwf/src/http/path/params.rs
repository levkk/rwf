//! Handles HTTP URL parameters, e.g. '/users/:id`.
use regex::Regex;
use std::collections::HashMap;

/// Handle URL parameters, e.g. `/api/orders/:id/create`.
#[derive(Debug)]
pub struct Params {
    params: HashMap<String, usize>,
    regex: Regex,
}

impl Params {
    /// Create new parameters handler.
    ///
    /// A handler requires a regex to extract parameters from the URL
    /// and the offsets for the captures in the regex where the parameters
    /// are expected to be.
    pub fn new(regex: Regex, params: HashMap<String, usize>) -> Self {
        Self { params, regex }
    }

    /// Extract a parameter from the URL.
    pub fn parameter<'a>(&'a self, base: &'a str, name: &str) -> Option<&'a str> {
        if let Some(index) = self.params.get(name) {
            let captures = self.regex.captures(base);

            if let Some(captures) = captures {
                if let Some(capture) = captures.get(*index) {
                    // TODO: figure out how to remove '/' from the regex capture.
                    let capture = capture.as_str();
                    return Some(if capture.starts_with("/") {
                        &capture[1..]
                    } else {
                        capture
                    });
                }
            }
        }

        None
    }

    /// Get the regex that parsed the parameters.
    pub fn regex(&self) -> &Regex {
        &self.regex
    }
}
