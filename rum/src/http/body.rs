use super::Error;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    Json,
    Text,
    Html,
    File(String),
}

pub struct Body {
    content: Vec<u8>,
    content_type: ContentType,
}

impl Body {
    pub fn json(value: impl Serialize) -> Result<Body, Error> {
        Ok(Body {
            content: serde_json::to_string(&value)?.as_bytes().to_vec(),
            content_type: ContentType::Json,
        })
    }

    pub fn html(value: impl ToString) -> Result<Body, Error> {
        Ok(Body {
            content: value.to_string().as_bytes().to_vec(),
            content_type: ContentType::Html,
        })
    }

    pub fn text(value: impl ToString) -> Result<Body, Error> {
        Ok(Body {
            content: value.to_string().as_bytes().to_vec(),
            content_type: ContentType::Text,
        })
    }
}
