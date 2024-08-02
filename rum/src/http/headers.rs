use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Headers {
    headers: HashMap<String, String>,
}

impl Headers {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: impl ToString, value: impl ToString) {
        self.headers
            .insert(name.to_string().to_lowercase(), value.to_string());
    }

    pub fn get(&self, name: &str) -> Option<&String> {
        self.headers.get(&name.to_lowercase())
    }

    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.headers.remove(&name.to_lowercase())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (name, value) in &self.headers {
            bytes.extend_from_slice(name.as_bytes());
            bytes.extend_from_slice(b": ");
            bytes.extend_from_slice(value.as_bytes());
            bytes.extend_from_slice(b"\r\n");
        }
        bytes
    }
}

impl From<HashMap<String, String>> for Headers {
    fn from(headers: HashMap<String, String>) -> Self {
        Self { headers }
    }
}
