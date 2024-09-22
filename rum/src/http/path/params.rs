use regex::Regex;

#[derive(Debug)]
pub struct Params {
    params: Vec<usize>,
    regex: Regex,
}

impl Params {
    pub fn new(regex: Regex, params: Vec<usize>) -> Self {
        Self { params, regex }
    }

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
