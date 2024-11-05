use crate::http::{self, Head, Headers, Method, Path, Version};
use worker::{Request, Response};

impl From<worker::Method> for Method {
    fn from(method: worker::Method) -> Method {
        match method {
            worker::Method::Get => Method::Get,
            worker::Method::Post => Method::Post,
            worker::Method::Head => Method::Head,
            worker::Method::Patch => Method::Patch,
            worker::Method::Put => Method::Put,
            worker::Method::Delete => Method::Delete,
            method => panic!("cf method conversion: {}", method),
        }
    }
}

pub struct CfRequest {
    request: http::Request,
}

impl CfRequest {
    pub async fn from_request(mut request: worker::Request) -> Result<Self, http::Error> {
        let url = request.url()?;
        let query = if let Some(query) = url.query() {
            format!("?{}", query)
        } else {
            "".to_string()
        };
        let path = format!("{}{}", url.path(), query);
        let path = Path::parse(&path)?;
        let version = Version::Http1;

        let mut headers = Headers::new();

        for (key, value) in request.headers() {
            headers.insert(key, value);
        }

        let head = Head::new(request.method().into(), path, version, headers);
        let body = request.bytes().await?;

        Ok(CfRequest {
            request: http::Request::new(head, &body)?,
        })
    }
}

impl From<super::Response> for Response {
    fn from(request: super::Response) -> Response {
        todo!()
    }
}
