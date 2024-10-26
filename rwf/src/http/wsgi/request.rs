use std::collections::HashMap;

use crate::http::{Error, Request, Response};
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

static WRAPPER: Lazy<Py<PyModule>> = Lazy::new(|| py_module!("uwsgi_wrapper.py"));

macro_rules! py_module {
    ($module:expr) => {
        Python::with_gil(|py| {
            let module: Py<PyModule> = PyModule::from_code_bound(
                py,
                include_str!($module),
                "uwsgi_wrapper.py",
                "uwsgi_wrapper",
            )
            .unwrap()
            .into();
            module
        })
    };
}

pub(crate) use py_module;

#[derive(Debug, Clone)]
pub struct WsgiRequest {
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl WsgiRequest {
    pub fn from_request(request: &Request) -> Result<Self, Error> {
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
        headers.insert("CONTENT_LENGTH".into(), body.len().to_string());
        headers.insert(
            "CONTENT_TYPE".into(),
            request
                .headers()
                .get("content-type")
                .unwrap_or(&"application/x-www-form-urlencoded".to_string())
                .clone(),
        );

        let wsgi = WsgiRequest { headers, body };

        Ok(wsgi)
    }

    pub fn send(self, application: &Py<PyAny>) -> Result<WsgiResponse, Error> {
        let body: Vec<Vec<u8>>;
        let code: String;
        let headers: Vec<(String, String)>;

        (body, code, headers) = Python::with_gil(|py| {
            let request = self.into_py(py);
            let wrapper: Py<PyAny> = WRAPPER.getattr(py, "wrapper").unwrap().into();
            let body: Py<PyAny> = wrapper.call1(py, (request, application)).unwrap();
            let (body, code, headers): (Vec<Vec<u8>>, String, Vec<(String, String)>) =
                body.extract(py).unwrap();

            (body, code, headers)
        });

        Ok(WsgiResponse {
            body,
            code,
            headers,
        })
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
        iter.push(("wsgi.input".into_py(py), body));
        IntoPyDict::into_py_dict_bound(iter, py).into()
    }
}

#[derive(Debug)]
pub struct WsgiResponse {
    code: String,
    headers: Vec<(String, String)>,
    body: Vec<Vec<u8>>,
}

impl WsgiResponse {
    pub fn to_response(self) -> Result<Response, Error> {
        let mut response = Response::new();
        let body = self.body.into_iter().flatten().collect::<Vec<u8>>();
        let body = String::from_utf8_lossy(&body);
        let code = self
            .code
            .split(" ")
            .next()
            .unwrap_or("200")
            .parse::<u16>()
            .unwrap_or(200);

        for (key, value) in self.headers {
            response = response.header(key, value);
        }

        Ok(response.html(body).code(code))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::http::request::test::dummy_request;

    static UWSGI_TEST: Lazy<Py<PyAny>> = Lazy::new(|| {
        Python::with_gil(|py| {
            let func: Py<PyAny> = PyModule::from_code_bound(
                py,
                "
def application(env, start_response):
    start_response('200 OK', [('Content-Type', 'text/plain')])
    return [b'Hello World']
",
                "application.py",
                "application",
            )
            .unwrap()
            .getattr("application")
            .unwrap()
            .into();

            func
        })
    });

    #[tokio::test]
    async fn test_wsgi_request() {
        let request = dummy_request().await.unwrap();
        let request = WsgiRequest::from_request(&request).unwrap();
        let application = Python::with_gil(|py| (*UWSGI_TEST).clone_ref(py));
        let response = request.send(&application).unwrap();

        assert_eq!(response.code, "200 OK");
        assert_eq!(response.headers[0].0, "Content-Type");
        assert_eq!(response.headers[0].1, "text/plain");
        let body = String::from_utf8_lossy(&response.body[0]);
        assert_eq!(body, "Hello World");
    }

    #[tokio::test]
    async fn test_django() {
        let request = dummy_request().await.unwrap();
        let _request = WsgiRequest::from_request(&request).unwrap();
    }
}
