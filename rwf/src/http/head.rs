//! Request head, including HTTP version and body.

use std::marker::Unpin;

use tokio::io::{AsyncRead, AsyncReadExt};

use super::{Authorization, Cookies, Error, Headers, Path, Query};
use crate::config::get_config;

/// HTTP method, e.g. GET, POST, etc.
#[derive(PartialEq, Clone, Debug, Default)]
pub enum Method {
    #[default]
    Get,
    Post,
    Put,
    Delete,
    Head,
    Patch,
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
            "PATCH" => Ok(Method::Patch),
            _ => Ok(Method::Other(value)),
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Method::*;

        let name = match self {
            Get => "GET".to_string(),
            Post => "POST".to_string(),
            Put => "PUT".to_string(),
            Delete => "DELETE".to_string(),
            Head => "HEAD".to_string(),
            Patch => "PATCH".to_string(),
            Other(other) => other.clone(),
        };

        write!(f, "{}", name)
    }
}

/// HTTP version, e.g. HTTP/1.1 or HTTP/2.
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

/// Request head.
#[derive(Debug, Clone, Default)]
pub struct Head {
    method: Method,
    path: Path,
    version: Version,
    headers: Headers,
}

impl Head {
    /// Read request head from a stream.
    pub async fn read(mut stream: impl AsyncRead + Unpin) -> Result<Self, Error> {
        let bytes_remaining = get_config().general.header_max_size; // avoid DDoS

        let request = Self::read_line(&mut stream, bytes_remaining)
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
            let header = Self::read_line(&mut stream, bytes_remaining).await?;
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

    pub fn authorization(&self) -> Option<Authorization> {
        Authorization::parse(match self.header("authorization") {
            Some(authorization) => authorization,
            None => return None,
        })
    }

    pub fn cookies(&self) -> Cookies {
        if let Some(cookie) = self.headers.get("cookie") {
            Cookies::parse(&cookie)
        } else {
            Cookies::default()
        }
    }

    /// Is this a HTTP/2 request?
    pub fn http2(&self) -> bool {
        self.version == Version::Http2
    }

    /// Is this a HTTP/1.1 request?
    pub fn http1(&self) -> bool {
        self.version == Version::Http1
    }

    /// Request path, including query parameters, e.g. `/foo?hello=world`.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// GET query.
    pub fn query(&self) -> &Query {
        self.path().query()
    }

    /// Request method, e.g. GET or POST.
    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn post(&self) -> bool {
        self.method() == &Method::Post
    }

    pub fn get(&self) -> bool {
        self.method() == &Method::Get
    }

    /// The size of the request body in bytes.
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

    /// Get the multipart boundary, if one exists.
    pub fn multipart_boundary(&self) -> Option<String> {
        if let Some(content_type) = self.headers.get("content-type") {
            let mut parts = content_type.split(";").into_iter();
            let _content_type = parts.next();
            if let Some(boundary) = parts.next() {
                let mut parts = boundary.split("=").into_iter();
                let _name = parts.next();
                if let Some(boundary) = parts.next() {
                    return Some(boundary.to_owned());
                }
            }
        }

        None
    }

    /// Request headers.
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Get a mutable reference to the headers.
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Get a header value by name, if it exists.
    ///
    /// Case insensitive.
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    /// Is the keep-alive flag set? Only for HTTP/1.1.
    /// HTTP/2 connections are keep-alive by design.
    pub fn keep_alive(&self) -> bool {
        self.http2()
            || self
                .headers
                .get("connection")
                .map(|s| s.to_lowercase() == "keep-alive")
                .unwrap_or(false)
    }

    /// Read a line from the stream, parsing out \r\n.
    async fn read_line(
        mut stream: impl AsyncRead + Unpin,
        mut bytes_remaining: usize,
    ) -> Result<String, std::io::Error> {
        let mut buf = Vec::new();
        let (mut cr, mut lf) = (false, false);

        while bytes_remaining > 0 {
            // stream should be buffered
            let b = stream.read_u8().await?;
            bytes_remaining -= 1;

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

    pub fn replace_path(&mut self, path: Path) {
        self.path = path.clone();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_read_line() {
        let line = b"Content-Type: application/json\r\n";
        let result = Head::read_line(&line[..], 4096).await.expect("read_line");
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
