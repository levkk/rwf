use base64::{engine::general_purpose, Engine as _};
use sha1::{Digest, Sha1};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc::{Receiver, Sender};

use super::Error;

use std::marker::Unpin;

pub mod message;
pub use message::Message;

#[derive(Debug, Clone)]
struct Request {
    key: String,
    version: String,
}

impl Request {
    pub fn from_http_request(request: super::Request) -> Result<Self, Error> {
        let key = match request.headers().get("sec-websocket-key") {
            Some(key) => key,
            None => return Err(Error::MalformedRequest("missing sec-websocket-key")),
        };

        let version = match request.headers().get("sec-websocket-version") {
            Some(version) => version,
            None => return Err(Error::MalformedRequest("sec-websocket-version")),
        };

        Ok(Self {
            key: key.to_string(),
            version: version.to_string(),
        })
    }
}

struct DataFrame {
    // header: u8,
}

pub struct Websocket<T> {
    stream: T,
}

impl<T> Websocket<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(stream: T) -> Self {
        Self { stream }
    }

    pub async fn handshake(mut self, request: super::Request) -> Result<Self, Error> {
        let request = Request::from_http_request(request)?;
        let accept = request.key.clone() + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
        let digest = Sha1::digest(accept.as_bytes());
        let base64 = general_purpose::STANDARD.encode(digest);

        super::Response::switching_protocols("websocket")
            .header("sec-websocket-accept", base64)
            .send(&mut self.stream)
            .await?;
        self.stream.flush().await?;

        Ok(self)
    }

    pub async fn handle(mut self) -> Result<(), Error> {
        loop {
            let header = self.stream.read_u8().await?;
            let fin = header & 0b10000000 == 128;
            let op_code = header & 0b00001111;
            let mask_len = self.stream.read_u8().await?;
            let masked = mask_len & 0b10000000 == 128;
            let len = mask_len & 0b01111111;

            let len = match len {
                0..=125 => len as u64,
                126 => {
                    let len = self.stream.read_u16().await?;
                    len as u64
                }

                127 => {
                    let len = self.stream.read_u64().await?;
                    len
                }

                _ => return Err(Error::MalformedRequest("websocket len")),
            };

            let mask = if masked {
                let mask = self.stream.read_u32().await?;
                [
                    ((mask & 0b11111111_00000000_00000000_00000000) >> 24) as u8,
                    ((mask & 0b00000000_11111111_00000000_00000000) >> 16) as u8,
                    ((mask & 0b00000000_00000000_11111111_00000000) >> 8) as u8,
                    (mask & 0b00000000_00000000_00000000_11111111) as u8,
                ]
            } else {
                [0, 0, 0, 0] // Not used, I just to return something.
            };

            let mut msg = vec![0u8; len as usize];
            self.stream.read_exact(&mut msg).await?;

            if masked {
                for i in 0..msg.len() {
                    msg[i] ^= mask[i % 4];
                }
            }
            println!("message: {:?}", String::from_utf8_lossy(&msg));
        }
    }
}
