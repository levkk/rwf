//! HTTP request.

use std::marker::Unpin;
use std::ops::Deref;
use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncReadExt};

use super::{Error, Head};

/// HTTP request.
///
/// The request is fully loaded into memory. It's safe to clone
/// since the contents are behind an [`std::sync::Arc`].
#[derive(Debug, Clone)]
pub struct Request {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    head: Head,
    body: Vec<u8>,
}

impl Request {
    /// Read the request in its entirety from a stream.
    pub async fn read(mut stream: impl AsyncRead + Unpin) -> Result<Self, Error> {
        let head = Head::read(&mut stream).await?;
        let content_length = head.content_length().unwrap_or(0);
        let mut body = vec![0u8; content_length];
        stream
            .read_exact(&mut body)
            .await
            .map_err(|_| Error::MalformedRequest("incorrect content length"))?;

        Ok(Request {
            inner: Arc::new(Inner { head, body }),
        })
    }

    /// Request's body as bytes.
    ///
    /// It's the job of the caller to handle encoding if any.
    pub fn body(&self) -> &[u8] {
        &self.inner.body
    }
}

impl Deref for Request {
    type Target = Head;

    fn deref(&self) -> &Self::Target {
        &self.inner.head
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_response() {
        let body = ("GET / HTTP/1.1\r\n".to_owned()
            + "Content-Type: application/json\r\n"
            + "Accept: */*\r\n"
            + "Content-Length: 4\r\n"
            + "\r\n"
            + "hello")
            .as_bytes()
            .to_vec();
        let response = Request::read(&body[..]).await.expect("response");
        println!("{:?}", response);
    }
}
