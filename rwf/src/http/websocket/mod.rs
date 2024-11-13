use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::Error;
use crate::view::TurboStream;

use std::marker::Unpin;

#[derive(Debug, Clone)]
pub struct Headers {
    pub key: String,
    pub version: String,
}

impl Headers {
    pub fn from_http_request(request: &super::Request) -> Result<Self, Error> {
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

#[derive(Debug)]
pub struct DataFrame {
    header: Header,
    meta: Meta,
    message: Option<Message>,
}

impl DataFrame {
    pub async fn read(stream: &mut (impl AsyncRead + Unpin)) -> Result<Self, Error> {
        let header = Header::read(stream).await?;
        let meta = Meta::read(stream).await?;
        let message = Message::read(&header, &meta, stream).await?;

        Ok(Self {
            header,
            meta,
            message: Some(message),
        })
    }

    pub async fn send(self, stream: &mut (impl AsyncWrite + Unpin)) -> Result<(), Error> {
        self.header.send(stream).await?;
        self.meta.send(stream).await?;

        if let Some(message) = self.message {
            message.send(stream).await?;
        }

        Ok(())
    }

    pub async fn flush(self, stream: &mut (impl AsyncWrite + Unpin)) -> Result<(), Error> {
        self.send(stream).await?;
        stream.flush().await?;

        Ok(())
    }

    pub fn is_pong(&self) -> bool {
        self.header.is_pong()
    }

    pub fn is_ping(&self) -> bool {
        self.header.is_ping()
    }

    pub fn new_pong(ping: DataFrame) -> Self {
        let meta = Meta {
            len: ping.message.as_ref().map(|m| m.len()).unwrap_or(0),
            mask: None,
        };
        let header = Header {
            fin: true,
            op_code: OpCode::Pong,
        };

        Self {
            header,
            meta,
            message: ping.message,
        }
    }

    pub fn new_ping() -> Self {
        Self {
            header: Header {
                fin: true,
                op_code: OpCode::Ping,
            },
            meta: Meta { len: 0, mask: None },
            message: None,
        }
    }

    pub fn message(self) -> Message {
        self.message.unwrap()
    }
}

#[derive(Debug, PartialEq)]
enum OpCode {
    Continuation,
    Text,
    Binary,
    Ping,
    Pong,
}

#[derive(Debug)]
struct Header {
    fin: bool,
    op_code: OpCode,
}

impl Header {
    async fn read(stream: &mut (impl AsyncRead + Unpin)) -> Result<Self, Error> {
        let header = stream.read_u8().await?;

        let fin = header & 0b10000000 == 128;
        let op_code = header & 0b00001111;

        let op_code = match op_code {
            0 => OpCode::Continuation,
            0x1 => OpCode::Text,
            0x2 => OpCode::Binary,
            0x9 => OpCode::Ping,
            0xA => OpCode::Pong,
            _ => return Err(Error::MalformedRequest("websocket control code")),
        };

        Ok(Self { fin, op_code })
    }

    async fn send(self, stream: &mut (impl AsyncWrite + Unpin)) -> Result<(), Error> {
        let mut byte: u8 = match self.op_code {
            OpCode::Continuation => 0,
            OpCode::Text => 0x1,
            OpCode::Binary => 0x2,
            OpCode::Ping => 0x9,
            OpCode::Pong => 0xA,
        };

        if self.fin {
            byte |= 0b10000000;
        }

        stream.write_u8(byte).await?;

        Ok(())
    }

    fn text(&self) -> bool {
        self.op_code == OpCode::Text
    }

    fn ping() -> Self {
        Self {
            fin: true,
            op_code: OpCode::Ping,
        }
    }

    fn is_pong(&self) -> bool {
        self.op_code == OpCode::Pong
    }

    fn is_ping(&self) -> bool {
        self.op_code == OpCode::Ping
    }
}

#[derive(Debug)]
struct Meta {
    len: usize,
    mask: Option<[u8; 4]>,
}

impl Meta {
    async fn read(stream: &mut (impl AsyncRead + Unpin)) -> Result<Self, Error> {
        let mask_len = stream.read_u8().await?;
        let masked = mask_len & 0b10000000 == 128;
        let len = mask_len & 0b01111111;

        let len = match len {
            0..=125 => len as u64,
            126 => {
                let mut len = [0u8; 2];
                stream.read_exact(&mut len).await?;
                u16::from_be_bytes(len) as u64
            }

            127 => {
                let mut len = [0u8; 8];
                stream.read_exact(&mut len).await?;
                u64::from_be_bytes(len) as u64
            }

            _ => return Err(Error::MalformedRequest("websocket len")),
        };

        let mask = if masked {
            let mask = stream.read_u32().await?;
            Some([
                ((mask & 0b11111111_00000000_00000000_00000000) >> 24) as u8,
                ((mask & 0b00000000_11111111_00000000_00000000) >> 16) as u8,
                ((mask & 0b00000000_00000000_11111111_00000000) >> 8) as u8,
                (mask & 0b00000000_00000000_00000000_11111111) as u8,
            ])
        } else {
            None
        };

        Ok(Self {
            len: len as usize,
            mask,
        })
    }

    fn len(&self) -> usize {
        self.len
    }

    fn mask(&self) -> &Option<[u8; 4]> {
        &self.mask
    }

    async fn send(self, stream: &mut (impl AsyncWrite + Unpin)) -> Result<(), Error> {
        let mut buf = vec![0u8; 0];

        let masked = if self.mask.is_some() {
            0b1000000
        } else {
            0b00000000
        };

        let _len = if self.len <= 125 {
            buf.push(self.len as u8 | masked);
        } else if self.len < u16::MAX as usize {
            buf.push(126 | masked);
            let bytes: [u8; 2] = u16::to_be_bytes(self.len as u16);
            buf.extend(&bytes);
        } else {
            let bytes: [u8; 8] = u64::to_be_bytes(self.len as u64);
            buf.push(127 | masked);
            buf.extend(&bytes);
        };

        stream.write_all(&buf).await?;
        if let Some(mask) = self.mask {
            stream.write_all(&mask).await?;
        }

        Ok(())
    }

    fn empty() -> Self {
        Meta { len: 0, mask: None }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
}

impl Message {
    pub fn turbo_stream(turbo_stream: TurboStream) -> Self {
        Message::Text(turbo_stream.render())
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Text(text) => text.as_bytes().len(),
            Self::Binary(bytes) => bytes.len(),
        }
    }

    fn op_code(&self) -> OpCode {
        match self {
            Self::Text(_) => OpCode::Text,
            _ => OpCode::Binary,
        }
    }

    async fn read(
        header: &Header,
        meta: &Meta,
        stream: &mut (impl AsyncRead + Unpin),
    ) -> Result<Self, Error> {
        let mut msg = vec![0u8; meta.len() as usize];

        stream.read_exact(&mut msg).await?;

        if let Some(mask) = meta.mask() {
            for i in 0..msg.len() {
                msg[i] ^= mask[i % 4];
            }
        }

        if header.text() {
            Ok(Self::Text(String::from_utf8_lossy(&msg).to_string()))
        } else {
            Ok(Self::Binary(msg))
        }
    }

    pub async fn send(&self, stream: &mut (impl AsyncWrite + Unpin)) -> Result<(), Error> {
        let header = Header {
            fin: true,
            op_code: self.op_code(),
        };

        let meta = Meta {
            len: self.len(),
            mask: None,
        };

        header.send(stream).await?;
        meta.send(stream).await?;

        match self {
            Self::Text(text) => stream.write_all(text.as_bytes()).await?,
            Self::Binary(bytes) => stream.write_all(bytes.as_slice()).await?,
        };

        stream.flush().await?;

        Ok(())
    }
}

pub trait ToMessage: Clone {
    fn to_message(self) -> Message;
}

impl ToMessage for Message {
    fn to_message(self) -> Message {
        self
    }
}

impl ToMessage for String {
    fn to_message(self) -> Message {
        Message::Text(self)
    }
}

impl ToMessage for &str {
    fn to_message(self) -> Message {
        Message::Text(self.to_string())
    }
}

impl ToMessage for Vec<u8> {
    fn to_message(self) -> Message {
        Message::Binary(self)
    }
}

impl ToMessage for &[u8] {
    fn to_message(self) -> Message {
        Message::Binary(self.to_vec())
    }
}

impl ToMessage for TurboStream {
    fn to_message(self) -> Message {
        Message::Text(self.render())
    }
}
