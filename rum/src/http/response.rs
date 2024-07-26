use std::collections::HashMap;
use std::marker::Unpin;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use serde::Serialize;

use super::Error;

#[derive(Debug, Clone)]
pub struct Response {
    code: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Response {
    pub fn new(code: u16, body: Vec<u8>) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Length".to_string(), body.len().to_string());
        Self {
            code,
            headers,
            body,
        }
    }

    pub fn not_found(body: &str) -> Self {
        Self::new(404, body.as_bytes().to_vec()).header("Content-Type", "text/html")
    }

    pub fn json(body: impl Serialize) -> Result<Self, Error> {
		let body = serde_json::to_vec(&body)?;
		Ok(Self::new(200, body).header("Content-Type", "application/json"))
	}

    pub fn header(mut self, name: impl ToString, value: impl ToString) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    pub async fn send(&self, mut stream: impl AsyncWrite + Unpin) -> Result<(), std::io::Error> {
        let mut response = format!("HTTP/1.1 {}\r\n", self.code);
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        response.push_str("\r\n");
        response.push_str(&String::from_utf8_lossy(&self.body));

        println!("{:?}", response);

        stream.write_all(response.as_bytes()).await
    }
}
