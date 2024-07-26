use super::{
    super::http::{Request, Response}, Error,
};
use crate::model::Model;
use std::collections::HashMap;
use std::future::Future;



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

#[derive(Clone)]
pub struct Route<F>
where
    F: Future<Output = Result<Response, Error>>,
{
    handler: fn(Request) -> F,
    method: Method,
    path: String,
}

impl<F> Route<F>
where
    F: Future<Output = Result<Response, Error>>,
{
    pub fn get(path: impl ToString, handler: fn(Request) -> F) -> Self {
        Route {
            handler,
            method: Method::GET,
            path: path.to_string(),
        }
    }

    pub async fn handle(&self, request: Request) -> Result<Response, Error> {
        (self.handler)(request).await
    }
}
