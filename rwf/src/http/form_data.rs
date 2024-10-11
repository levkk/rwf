use super::{Error, Query, Request};
use std::str::FromStr;

#[derive(Clone)]
pub enum FormData {
    UrlEncoded(Query),
}

impl FormData {
    pub fn from_request(request: &Request) -> Result<Self, Error> {
        let content_type = request
            .header("content-type")
            .ok_or(Error::MalformedRequest("content-type header is required"))?;

        if content_type.contains("application/x-www-form-urlencoded") {
            Self::from_url_encoded(request)
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
        }
    }

    pub fn get_required<T: FromStr>(&self, name: &str) -> Result<T, Error> {
        match self.get(name) {
            Some(v) => Ok(v),
            None => Err(Error::MissingParameter),
        }
    }
}
