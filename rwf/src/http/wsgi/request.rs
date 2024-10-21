use std::collections::HashMap;

use crate::http::{Error, Request};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

use once_cell::sync::Lazy;

static INTO_BYTES: Lazy<Py<PyAny>> = Lazy::new(|| {
    Python::with_gil(|py| {
        let fun: Py<PyAny> = PyModule::from_code_bound(
            py,
            "
def into_bytes(b):
    from io import BytesIO
    return BytesIO(bytes(b))
",
            "into_bytes.py",
            "into_bytes",
        )
        .unwrap()
        .getattr("into_bytes")
        .unwrap()
        .into();

        fun
    })
});

#[derive(Debug)]
pub struct WsgiRequest {
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl WsgiRequest {
    fn from_request(request: &Request) -> Result<Self, Error> {
        let map = request.headers().clone().into_raw();
        let body = request.body().to_vec();

        let mut headers = HashMap::new();
        for (key, value) in map {
            headers.insert(format!("HTTP_{}", key.to_uppercase()), value);
        }

        headers.insert("REQUEST_METHOD".into(), request.method().to_string());
        headers.insert("PATH_INFO".into(), request.path().base().to_owned());
        headers.insert("REQUEST_URI".into(), request.path().to_string());
        headers.insert(
            "QUERY_STRING".into(),
            request.path().query().to_string().replace("?", ""), // Remove the leading ?
        );
        headers.insert("SERVER_PROTOCOL".into(), "HTTP/1.1".into());
        headers.insert("UWSGI_ROUTER".into(), "http".into());
        headers.insert("REMOTE_ADDR".into(), request.peer().ip().to_string());
        headers.insert("REMOTE_PORT".into(), request.peer().port().to_string());

        let wsgi = WsgiRequest { headers, body };

        Ok(wsgi)
    }
}

impl IntoPy<PyObject> for WsgiRequest {
    fn into_py(self, py: Python<'_>) -> PyObject {
        let mut iter = self
            .headers
            .into_iter()
            .map(|(k, v)| (k.into_py(py), v.into_py(py)))
            .collect::<Vec<_>>();

        let body = Python::with_gil(|py| INTO_BYTES.call1(py, (self.body,)).unwrap());
        // let body: Py<PyAny> = self.body.into_py(py).into();
        iter.push(("wsgi.input".into_py(py), body));
        IntoPyDict::into_py_dict_bound(iter, py).into()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::http::request::test::dummy_request;

    #[tokio::test]
    async fn test_wsgi_request() {
        let request = dummy_request().await.unwrap();
        let request = WsgiRequest::from_request(&request).unwrap();
        Python::with_gil(|py| {
            let fun: Py<PyAny> = PyModule::from_code_bound(
                py,
                "
def debug(env):
    print(env)
",
                "request.py",
                "request",
            )
            .unwrap()
            .getattr("debug")
            .unwrap()
            .into();

            fun.call1(py, (request,)).unwrap();
        });
    }
}
