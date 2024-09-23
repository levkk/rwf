use regex::Regex;

/// Handle URL parameters, e.g. `/api/orders/:id/create`.
#[derive(Debug)]
pub struct Params {
    params: Vec<usize>,
    regex: Regex,
}

impl Params {
    /// Create new parameters handler.
    ///
    /// A handler requires a regex to extract parameters from the URL
    /// and the offsets for the captures in the regex where the parameters
    /// are expected to be.
    pub fn new(regex: Regex, params: Vec<usize>) -> Self {
        Self { params, regex }
    }

    /// Extract a parameter from the URL.
    pub fn parameter<'a>(&'a self, base: &'a str, index: usize) -> Option<&'a str> {
        if let Some(index) = self.params.get(index) {
            let captures = self.regex.captures(base);

            if let Some(captures) = captures {
                if let Some(capture) = captures.get(*index) {
                    return Some(capture.as_str());
                }
            }
        }

        None
    }

    pub fn regex(&self) -> &Regex {
        &self.regex
    }
}
