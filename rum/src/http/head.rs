use std::collections::HashMap;
use std::marker::Unpin;

use tokio::io::{AsyncRead, AsyncReadExt};

use super::{Error, Headers, Path};

#[derive(PartialEq, Clone, Debug, Default)]
pub enum Method {
    #[default]
    Get,
    Post,
    Put,
    Delete,
    Head,
    Other(String),
}

impl TryFrom<String> for Method {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "HEAD" => Ok(Method::Head),
            _ => Ok(Method::Other(value)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Version {
    #[default]
    Http1,
    Http2,
    Unknown,
}

impl TryFrom<String> for Version {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "HTTP/1.1" => Ok(Version::Http1),
            "HTTP/2" => Ok(Version::Http2),
            _ => Ok(Version::Unknown),
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Version::Http1 => write!(f, "HTTP/1.1"),
            Version::Http2 => write!(f, "HTTP/2"),
            Version::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Head {
    method: Method,
    path: Path,
    version: Version,
    headers: Headers,
}

impl Head {
    pub async fn read(mut stream: impl AsyncRead + Unpin) -> Result<Self, Error> {
        let request = Self::read_line(&mut stream)
            .await?
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let method = request
            .get(0)
            .ok_or(Error::MalformedRequest("method"))?
            .to_string();
        let method = Method::try_from(method)?;

        let path = request.get(1).ok_or(Error::MalformedRequest("path"))?;
        let path = Path::parse(path)?;

        let version = request
            .get(2)
            .ok_or(Error::MalformedRequest("version"))?
            .to_string();
        let version = Version::try_from(version)?;

        let mut headers = Headers::new();

        loop {
            let header = Self::read_line(&mut stream).await?;
            if header.is_empty() {
                break;
            } else {
                let header = header
                    .split(":")
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>();
                let name = header
                    .get(0)
                    .ok_or(Error::MalformedRequest("header name"))?
                    .to_lowercase();
                let value = header
                    .get(1)
                    .ok_or(Error::MalformedRequest("header value"))?
                    .clone();
                headers.insert(name, value);
            }
        }

        Ok(Head {
            method,
            path,
            version,
            headers,
        })
    }

    pub fn http2(&self) -> bool {
        self.version == Version::Http2
    }

    pub fn http1(&self) -> bool {
        self.version == Version::Http1
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn content_length(&self) -> Option<usize> {
        if let Some(cl) = self.headers.get("content-length") {
            if let Ok(cl) = cl.parse::<usize>() {
                Some(cl)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    pub fn keep_alive(&self) -> bool {
        self.headers
            .get("connection")
            .map(|s| s.to_lowercase() == "keep-alive")
            .unwrap_or(false)
    }

    async fn read_line(mut stream: impl AsyncRead + Unpin) -> Result<String, std::io::Error> {
        let mut buf = Vec::new();
        let (mut cr, mut lf) = (false, false);

        loop {
            let b = stream.read_u8().await?;

            if b == '\r' as u8 {
                cr = true;
            } else if b == '\n' as u8 {
                lf = true;
            } else {
                buf.push(b);
            }

            if cr && lf {
                break;
            }
        }

        Ok(String::from_utf8_lossy(&buf).to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_read_line() {
        let line = b"Content-Type: application/json\r\n";
        let result = Head::read_line(&line[..]).await.expect("read_line");
        assert_eq!(result, "Content-Type: application/json");
    }

    #[tokio::test]
    async fn test_parse_header() {
        let body = ("GET / HTTP/1.1\r\n".to_owned()
            + "Content-Type: application/json\r\n"
            + "Accept: */*\r\n"
            + "Content-Length: 4\r\n"
            + "\r\n"
            + "hello")
            .as_bytes()
            .to_vec();
        let head = Head::read(&body[..]).await.expect("head");
        assert!(head.http1());
        assert_eq!(head.method(), &Method::Get);
        assert_eq!(head.path().path(), "/");
        assert_eq!(head.content_length(), Some(4));
    }
}
