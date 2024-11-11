use super::{Error, Query, Request};
use std::str::FromStr;

use std::collections::hash_map::{HashMap, IntoIter};

#[derive(Clone, Debug)]
pub enum FormData {
    UrlEncoded(Query),
    Multipart(Multipart),
}

impl FormData {
    /// Extract form data from request.
    pub fn from_request(request: &Request) -> Result<Self, Error> {
        let content_type = request
            .header("content-type")
            .ok_or(Error::MalformedRequest("content-type header is required"))?;

        if content_type.contains("application/x-www-form-urlencoded") {
            Self::from_url_encoded(request)
        } else if content_type.contains("multipart/form-data") {
            if let Some(boundary) = content_type.split(";").last() {
                let boundary = boundary.split("=").last();
                if let Some(boundary) = boundary {
                    let multipart = Multipart::read(request.body(), boundary.trim())?;

                    Ok(Self::Multipart(multipart))
                } else {
                    Err(Error::MalformedRequest("multipart missing boundary"))
                }
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

    /// Get a value from the form submission.
    pub fn get<T: FromStr>(&self, name: &str) -> Option<T> {
        match self {
            FormData::UrlEncoded(query) => query.get::<T>(name),
            FormData::Multipart(multipart) => {
                let entry = multipart.get(name);
                if let Some(entry) = entry {
                    if entry.content_disposition.filename.is_none() {
                        if let Ok(s) = entry.to_string() {
                            match T::from_str(&s) {
                                Ok(s) => Some(s),
                                Err(_) => None,
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Get file data from a multipart form.
    pub fn file<'a>(&'a self, name: &str) -> Option<File<'a>> {
        match self {
            FormData::Multipart(multipart) => multipart.get(name).map(|f| File {
                body: f.as_bytes(),
                name: f
                    .content_disposition
                    .filename
                    .clone()
                    .unwrap_or("".to_string()),
            }),
            _ => None,
        }
    }

    /// An owning iterator over the form data.
    pub fn into_iter(self) -> IntoIter<String, String> {
        match self {
            FormData::UrlEncoded(query) => query.into_iter(),
            FormData::Multipart(multipart) => {
                let entries = multipart
                    .entries
                    .into_iter()
                    .filter(|entry| entry.1.content_disposition.filename.is_none())
                    .map(|entry| (entry.0, entry.1.to_string().unwrap_or("".to_string())))
                    .into_iter()
                    .collect::<HashMap<String, String>>();
                entries.into_iter()
            }
        }
    }

    /// Return a Result instead of option for the required parameter.
    pub fn get_required<T: FromStr>(&self, name: &str) -> Result<T, Error> {
        match self.get(name) {
            Some(v) => Ok(v),
            None => Err(Error::MissingParameter),
        }
    }
}

/// Multipart for submission.
#[derive(Debug, Clone)]
pub struct Multipart {
    entries: HashMap<String, MultipartEntry>,
}

/// Multipart form submission entry.
#[derive(Debug, Clone)]
pub struct MultipartEntry {
    data: Vec<u8>,
    content_disposition: ContentDisposition,
    content_type: Option<String>,
}

impl MultipartEntry {
    /// Convert the multipart entry to string, if it's valid UTF-8 data.
    pub fn to_string(&self) -> Result<String, Error> {
        Ok(String::from_utf8(self.data.clone())?)
    }

    /// Get the multipart entry as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the Content-Type header passed in the multipart form
    /// for this entry.
    pub fn content_type(&self) -> Option<String> {
        if let Some(ref content_type) = self.content_type {
            content_type.split(":").last().map(|s| s.trim().to_string())
        } else {
            None
        }
    }
}

// Read a single "HTTP line".
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

/// A file uploaded via a multipart form.
#[derive(Debug, Clone)]
pub struct File<'a> {
    /// File data.
    pub body: &'a [u8],
    /// File name provided by the browser.
    pub name: String,
}

impl Multipart {
    /// Read multi-part body from request's body.
    fn read(body: &[u8], boundary: &str) -> Result<Self, Error> {
        let mut entries = HashMap::new();
        let mut reader = body.into_iter();

        let start_boundary = format!("--{}", boundary).as_bytes().to_vec();
        let end_boundary = format!("--{}--", boundary).as_bytes().to_vec();

        let mut buf = Vec::new();
        let mut content_disposition: Option<ContentDisposition> = None;
        let mut content_type: Option<String> = None;

        loop {
            let line = read_line!(reader);

            if line.is_empty() {
                break;
            }

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
                let ct = ContentDisposition::parse(&String::from_utf8(read_line!(reader))?)?;

                // Check if we're parsing a file.
                if ct.filename.is_some() {
                    // Get content type.
                    let ct = read_line!(reader);
                    if ct.to_ascii_lowercase().starts_with(b"content-type") {
                        content_type = Some(String::from_utf8(ct)?);
                        // Read and discard "\r\n".
                        let _ = read_line!(reader);
                    }
                } else {
                    // Read and discard "\r\n".
                    let _ = read_line!(reader);
                }
                content_disposition = Some(ct);
            } else if line == end_boundary {
                // We've reached the end of the form data.
                if let Some(content_disposition) = content_disposition.take() {
                    entries.insert(
                        content_disposition.name.clone(),
                        MultipartEntry {
                            data: buf.clone(),
                            content_disposition,
                            content_type: content_type.take(),
                        },
                    );
                }
                break;
            } else {
                buf.extend(line);
            }
        }

        Ok(Multipart { entries })
    }

    /// Get a multi-part entry, if it exists.
    pub fn get(&self, name: &str) -> Option<&MultipartEntry> {
        self.entries.get(name)
    }
}

/// Content-Disposition header.
#[derive(Debug, Clone)]
pub struct ContentDisposition {
    pub name: String,
    pub filename: Option<String>,
}

impl ContentDisposition {
    // Parse the Content-Disposition header.
    fn parse(header: &str) -> Result<ContentDisposition, Error> {
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

        let mp = Multipart::read(multipart.as_bytes(), "ExampleBoundaryString").unwrap();

        assert_eq!(mp.entries.len(), 2);
        assert_eq!(
            mp.get("description").unwrap().to_string().unwrap(),
            "Description input value"
        );
        assert_eq!(
            mp.get("myFile").unwrap().as_bytes(),
            b"[content of the file foo.txt chosen by the user]"
        );
        assert_eq!(
            mp.get("myFile").unwrap().content_type().unwrap(),
            "text/plain"
        );

        let req = format!(
            "POST /upload HTTP/1.1\r\nContent-Length: {}\r\nContent-Type: multipart/form-data; boundary=ExampleBoundaryString\r\n\r\n{}",
            multipart.len(),
            multipart,
        )
        .as_bytes()
        .to_vec();

        let peer = "127.0.0.1:6000".parse().unwrap();
        let request = Request::read(peer, &req[..]).await.unwrap();
        let form_data = request.form_data().unwrap();
        let file = form_data.file("myFile").unwrap();
        assert_eq!(file.name, "foo.txt");
        assert_eq!(
            file.body,
            b"[content of the file foo.txt chosen by the user]"
        );
        let input = form_data.get::<String>("description").unwrap();
        assert_eq!(input, "Description input value");
    }
}
