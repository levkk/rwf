

use hyper::StatusCode;


use std::collections::HashMap;




#[derive(Debug, Clone, Default)]
pub struct Response {
    body: Vec<u8>,
    headers: HashMap<String, String>,
    status: StatusCode,
}

impl Response {}
