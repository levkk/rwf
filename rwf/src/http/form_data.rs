use super::{Error, Query, Request};
use std::str::FromStr;

use std::collections::hash_map::{HashMap, IntoIter};

#[derive(Clone)]
pub enum FormData {
    UrlEncoded(Query),
    Multipart(Multipart),
}

impl FormData {
    pub fn from_request(request: &Request) -> Result<Self, Error> {
        let content_type = request
            .header("content-type")
            .ok_or(Error::MalformedRequest("content-type header is required"))?;

        if content_type.contains("application/x-www-form-urlencoded") {
            Self::from_url_encoded(request)
        } else if content_type.contains("multipart/form-data") {
            if let Some(boundary) = content_type.split(";").last() {
                let multipart = Multipart::read(request.body(), boundary.trim())?;

                Ok(Self::Multipart(multipart))
            } else {
                Err(Error::MalformedRequest("multipart missing boundary"))
            }
        } else {
            return Err(Error::MalformedRequest(
                "only www-url-encoded form is currently supported",
            ));
        }
    }

    fn from_url_encoded(request: &Request) -> Result<Self, Error> {
        Ok(Self::UrlEncoded(Query::parse(&request.string())))
    }

    pub fn get<T: FromStr>(&self, name: &str) -> Option<T> {
        match self {
            FormData::UrlEncoded(query) => query.get::<T>(name),
            FormData::Multipart(_) => None,
        }
    }

    /// An owning iterator over the form data.
    pub fn into_iter(self) -> IntoIter<String, String> {
        match self {
            FormData::UrlEncoded(query) => query.into_iter(),
            _ => todo!(),
        }
    }

    pub fn get_required<T: FromStr>(&self, name: &str) -> Result<T, Error> {
        match self.get(name) {
            Some(v) => Ok(v),
            None => Err(Error::MissingParameter),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Multipart {
    entries: HashMap<String, MultipartEntry>,
}

#[derive(Debug, Clone)]
pub struct MultipartEntry {
    data: Vec<u8>,
    content_disposition: ContentDisposition,
    content_type: Option<String>,
}

impl MultipartEntry {
    pub fn to_string(&self) -> Result<String, Error> {
        Ok(String::from_utf8(self.data.clone())?)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn content_type(&self) -> Option<String> {
        if let Some(ref content_type) = self.content_type {
            content_type.split(":").last().map(|s| s.trim().to_string())
        } else {
            None
        }
    }
}

macro_rules! read_line {
    ($reader: expr) => {{
        let mut buf = Vec::new();

        while let Some(c) = $reader.next() {
            if *c as char == '\r' {
                let _ = $reader.next();
                break;
            } else {
                buf.push(*c);
            }
        }

        buf
    }};
}

impl Multipart {
    pub fn read(body: &[u8], boundary: &str) -> Result<Self, Error> {
        let mut entries = HashMap::new();
        let mut reader = body.into_iter();

        let start_boundary = format!("--{}", boundary).as_bytes().to_vec();
        let end_boundary = format!("--{}--", boundary).as_bytes().to_vec();

        let mut buf = Vec::new();
        let mut content_disposition: Option<ContentDisposition> = None;
        let mut content_type: Option<String> = None;

        loop {
            let line = read_line!(reader);

            if line == start_boundary {
                if let Some(content_disposition) = content_disposition.take() {
                    entries.insert(
                        content_disposition.name.clone(),
                        MultipartEntry {
                            data: buf.clone(),
                            content_disposition,
                            content_type: content_type.take(),
                        },
                    );
                    buf.clear();
                }
                let ct = String::from_utf8(read_line!(reader))?;
                content_disposition = Some(ContentDisposition::parse(&ct)?);

                let ct = read_line!(reader);
                if ct.to_ascii_lowercase().starts_with(b"content-type") {
                    content_type = Some(String::from_utf8(ct)?);
                    let _ = read_line!(reader);
                }
            } else if line == end_boundary {
                if let Some(content_disposition) = content_disposition.take() {
                    entries.insert(
                        content_disposition.name.clone(),
                        MultipartEntry {
                            data: buf.clone(),
                            content_disposition,
                            content_type: content_type.take(),
                        },
                    );
                    buf.clear();
                }
                break;
            } else {
                buf.extend(line);
            }
        }

        Ok(Multipart { entries })
    }

    pub fn get(&self, name: &str) -> Option<&MultipartEntry> {
        self.entries.get(name)
    }
}

#[derive(Debug, Clone)]
pub struct ContentDisposition {
    pub name: String,
    pub filename: Option<String>,
}

impl ContentDisposition {
    pub fn parse(header: &str) -> Result<ContentDisposition, Error> {
        let mut names = header.split(":").into_iter().map(|s| s.trim());

        if let Some(header) = names.next() {
            if header.to_lowercase() != "content-disposition" {
                return Err(Error::MalformedRequest(
                    "content-disposition header is missing",
                ));
            }
        }

        if let Some(params) = names.next() {
            let mut params = params.split(";").into_iter().map(|s| s.trim());
            let _form_data = params.next();

            let mut content_name: Option<String> = None;
            let mut filename: Option<String> = None;

            for param in params {
                let mut parts = param.split("=").into_iter();
                let name = parts.next();
                let value = parts.next();

                if let Some(name) = name {
                    match name {
                        "name" => {
                            if let Some(value) = value {
                                content_name = Some(value.replace("\"", ""));
                            }
                        }

                        "filename" => {
                            if let Some(value) = value {
                                filename = Some(value.replace("\"", ""));
                            }
                        }

                        _ => (),
                    }
                }
            }

            if let Some(name) = content_name {
                return Ok(ContentDisposition { name, filename });
            }
        }

        Err(Error::MalformedRequest("multipart/form-data is malformed"))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_content_disposition_header() {
        let header = r#"Content-Disposition: form-data; name="description""#;
        let header = ContentDisposition::parse(header).unwrap();
        assert_eq!(header.name, "description");

        let header = r#"Content-Disposition: form-data; name="myFile"; filename="foo.txt""#;
        let header = ContentDisposition::parse(header).unwrap();
        assert_eq!(header.name, "myFile");
        assert_eq!(header.filename, Some("foo.txt".to_string()));
    }

    #[tokio::test]
    async fn test_multipart() {
        let multipart = r#"--ExampleBoundaryString
Content-Disposition: form-data; name="description"

Description input value
--ExampleBoundaryString
Content-Disposition: form-data; name="myFile"; filename="foo.txt"
Content-Type: text/plain

[content of the file foo.txt chosen by the user]
--ExampleBoundaryString--
"#
        .split("\n")
        .into_iter()
        .map(|s| format!("{}\r\n", s))
        .collect::<String>();

        let multipart = Multipart::read(multipart.as_bytes(), "ExampleBoundaryString").unwrap();

        assert_eq!(multipart.entries.len(), 2);
        assert_eq!(
            multipart.get("description").unwrap().to_string().unwrap(),
            "Description input value"
        );
        assert_eq!(
            multipart.get("myFile").unwrap().as_bytes(),
            b"[content of the file foo.txt chosen by the user]"
        );
        assert_eq!(
            multipart.get("myFile").unwrap().content_type().unwrap(),
            "text/plain"
        );
    }
}
