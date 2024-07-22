use super::{Error, Request, Response};
use std::collections::HashMap;

// use hyper::Request;
use http::Method;

#[derive(Debug, Clone)]
pub struct Query {
    values: HashMap<String, String>,
}

impl TryFrom<&str> for Query {
    type Error = Error;

    fn try_from(query: &str) -> Result<Query, Error> {
        let mut values = HashMap::new();
        let kvs = query.split("&");
        for kv in kvs {
            let mut pair = kv.split("=");
            if let Some(key) = pair.next() {
                if let Some(value) = pair.next() {
                    values.insert(key.to_owned(), value.to_owned());
                } else {
                    values.insert(key.to_owned(), "".to_owned());
                }
            }
        }

        Ok(Query { values })
    }
}

impl<'a> Query {
    pub fn get(&'a self, key: &str) -> Option<&'a String> {
        self.values.get(key)
    }

    pub fn exists(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}

#[derive(Debug)]
pub struct Route {
    path: String,
    method: Method,
    handler: fn(Request) -> Result<Response, Error>,
}

impl Route {
    pub fn matches(&self, request: &hyper::Request<hyper::body::Incoming>) -> bool {
        let path = request.uri().path();
        self.method == request.method() && self.path == path
    }
}
