use libc::uintptr_t;
use once_cell::sync::OnceCell;
use std::ffi::{c_char, c_int, CStr, CString};
use std::fs::canonicalize;
use std::mem::MaybeUninit;
use std::path::Path;

use std::collections::HashMap;
use tracing::info;

// Make sure the Ruby VM is initialized only once.
static RUBY_INIT: OnceCell<Ruby> = OnceCell::new();

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RackResponse {
    pub value: uintptr_t,
    pub code: c_int,
    pub num_headers: c_int,
    pub headers: *mut KeyValue,
    pub body: *mut c_char,
    pub is_file: c_int,
}

#[repr(C)]
#[derive(Debug)]
pub struct KeyValue {
    key: *const c_char,
    value: *const c_char,
}

#[repr(C)]
#[derive(Debug)]
pub struct RackRequest {
    env: *const KeyValue,
    length: c_int,
}

impl RackRequest {
    pub fn send(env: HashMap<String, String>) -> Result<RackResponse, Error> {
        // let mut c_strings = vec![];
        let mut keys = vec![];

        let (mut k, mut v) = (vec![], vec![]);

        for (key, value) in &env {
            let key = CString::new(key.as_str()).unwrap();
            let value = CString::new(value.as_str()).unwrap();
            k.push(key);
            v.push(value);

            let env_key = KeyValue {
                key: k.last().unwrap().as_ptr(),
                value: v.last().unwrap().as_ptr(),
            };

            keys.push(env_key);
        }

        let req = RackRequest {
            length: keys.len() as c_int,
            env: keys.as_ptr(),
        };

        // Hardcoded to Rails, but can be any other Rack app.
        let app_name = CString::new("Rails.application").unwrap();

        let mut response: RackResponse = unsafe { MaybeUninit::zeroed().assume_init() };

        let result = unsafe { rwf_app_call(req, app_name.as_ptr(), &mut response) };

        if result != 0 {
            return Err(Error::App);
        } else {
            Ok(response)
        }
    }
}

/// RackResponse with values allocated in Rust memory space.
#[derive(Debug)]
pub struct RackResponseOwned {
    code: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    is_file: bool,
}

impl RackResponseOwned {
    /// Request body.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Request HTTP code.
    pub fn code(&self) -> u16 {
        self.code
    }

    /// Is the request a file?
    pub fn is_file(&self) -> bool {
        self.is_file
    }

    /// Request headers.
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
}

impl From<RackResponse> for RackResponseOwned {
    /// Move all data out of C into Rust-owned memory.
    /// This also drops the reference to the Rack response array,
    /// allowing it to be garbage collected.
    fn from(response: RackResponse) -> RackResponseOwned {
        let code = response.code as u16;

        let mut headers = HashMap::new();

        for n in 0..response.num_headers {
            let env_key = unsafe { response.headers.offset(n as isize) };
            let name = unsafe { CStr::from_ptr((*env_key).key) };
            let value = unsafe { CStr::from_ptr((*env_key).value) };

            // Headers should be valid UTF-8.
            headers.insert(
                name.to_string_lossy().to_string(),
                value.to_string_lossy().to_string(),
            );
        }

        // Body can be anything.
        let body = unsafe { CStr::from_ptr(response.body) };
        let body = Vec::from(body.to_bytes());

        RackResponseOwned {
            code,
            headers,
            body,
            is_file: response.is_file == 1,
        }
    }
}

impl RackResponse {
    /// Parse the Rack response from a Ruby value.
    pub fn new(value: &Value) -> Self {
        unsafe { rwf_rack_response_new(value.raw_ptr()) }
    }
}

impl Drop for RackResponse {
    fn drop(&mut self) {
        unsafe { rwf_rack_response_drop(self) }
    }
}

#[link(name = "ruby")]
extern "C" {
    fn ruby_cleanup(code: c_int) -> c_int;
    fn rb_errinfo() -> uintptr_t;

    // Execute some Ruby code.
    fn rb_eval_string_protect(code: *const c_char, state: *mut c_int) -> uintptr_t;
    fn rb_obj_as_string(value: uintptr_t) -> uintptr_t;

    fn rb_gc_disable() -> c_int;
    fn rb_gc_enable() -> c_int;
}

#[link(name = "rwf_ruby")]
extern "C" {
    /// Get the type of the object.
    fn rwf_rb_type(value: uintptr_t) -> c_int;

    /// Get the CStr value. Careful with this one,
    /// if the object isn't a string, this will segfault.
    fn rwf_value_cstr(value: uintptr_t) -> *mut c_char;

    /// Clear error state after handling an exception.
    fn rwf_clear_error_state();

    /// Convert the Rack response to a struct we can work with.
    /// The Rack response is an array of three elements:
    /// - HTTP code (int)
    /// - headers (Hash)
    /// - body (String)
    fn rwf_rack_response_new(value: uintptr_t) -> RackResponse;

    /// Deallocate memory allocated for converting the Rack response
    /// from Ruby to Rust.
    fn rwf_rack_response_drop(response: &RackResponse);

    /// Load an app into the VM.
    fn rwf_load_app(path: *const c_char) -> c_int;

    /// Initialize Ruby correctly.
    fn rwf_init_ruby();

    fn rwf_app_call(
        request: RackRequest,
        app_name: *const c_char,
        response: *mut RackResponse,
    ) -> c_int;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Ruby VM did not start")]
    VmInit,

    #[error("{err}")]
    Eval { err: String },

    #[error("Ruby app failed to load")]
    App,
}

#[derive(Debug)]
pub struct Value {
    ptr: uintptr_t,
}

#[derive(Debug, PartialEq)]
#[repr(C)]
pub enum Type {
    None = 0x00,

    Object = 0x01,
    Class = 0x02,
    Module = 0x03,
    Float = 0x04,
    RString = 0x05,
    Regexp = 0x06,
    Array = 0x07,
    Hash = 0x08,
    Struct = 0x09,
    Bignum = 0x0a,
    File = 0x0b,
    Data = 0x0c,
    Match = 0x0d,
    Complex = 0x0e,
    Rational = 0x0f,

    Nil = 0x11,
    True = 0x12,
    False = 0x13,
    Symbol = 0x14,
    Fixnum = 0x15,
    Undef = 0x16,

    IMemo = 0x1a,
    Node = 0x1b,
    IClass = 0x1c,
    Zombie = 0x1d,

    Mask = 0x1f,
}

impl Value {
    pub fn to_string(&self) -> String {
        if self.ty() == Type::RString {
            unsafe {
                let cstr = rwf_value_cstr(self.ptr);
                CStr::from_ptr(cstr).to_string_lossy().to_string()
            }
        } else {
            String::new()
        }
    }

    pub fn ty(&self) -> Type {
        let ty = unsafe { rwf_rb_type(self.ptr) };
        match ty {
            0x05 => Type::RString,
            _ => Type::Nil,
        }
    }

    pub fn raw_ptr(&self) -> uintptr_t {
        self.ptr
    }
}

impl From<uintptr_t> for Value {
    fn from(ptr: uintptr_t) -> Value {
        Value { ptr }
    }
}

pub struct Ruby;

impl Ruby {
    pub fn init() -> Result<(), Error> {
        RUBY_INIT.get_or_try_init(move || Ruby::new())?;

        Ok(())
    }

    fn new() -> Result<Self, Error> {
        unsafe {
            rwf_init_ruby();
            Ok(Ruby {})
        }
    }

    /// Preload the Rack app into memory. Run this before trying to run anything else.
    pub fn load_app(path: impl AsRef<Path> + Copy) -> Result<(), Error> {
        Self::init()?;
        let path = path.as_ref();

        let version = Self::eval("RUBY_VERSION").unwrap().to_string();
        info!("Using {}", version);

        if path.exists() {
            // We use `require`, which only works with abslute paths.
            let absolute = canonicalize(path).unwrap();
            let s = absolute.display().to_string();
            let cs = CString::new(s).unwrap();

            unsafe {
                if rwf_load_app(cs.as_ptr()) != 0 {
                    return Err(Error::App);
                }
            }
        }

        Ok(())
    }

    /// Run some Ruby code. If an exception is thrown, return the error.
    pub fn eval(code: &str) -> Result<Value, Error> {
        Self::init()?;

        unsafe {
            let mut state: c_int = 0;
            let c_string = CString::new(code).unwrap();
            let value = rb_eval_string_protect(c_string.as_ptr(), &mut state);

            if state != 0 {
                let err = rb_errinfo();
                let err = Value::from(rb_obj_as_string(err)).to_string();
                rwf_clear_error_state();

                Err(Error::Eval { err })
            } else {
                Ok(Value { ptr: value })
            }
        }
    }

    pub fn gc_disable() {
        unsafe {
            rb_gc_disable();
        }
    }

    pub fn gc_enable() {
        unsafe {
            rb_gc_enable();
        }
    }
}

impl Drop for Ruby {
    fn drop(&mut self) {
        unsafe {
            ruby_cleanup(0);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rack_response() {
        let response = Ruby::eval(r#"[200, {"hello": "world", "the year is 2024": "linux desktop is coming"}, ["apples and oranges"]]"#).unwrap();
        let response = RackResponse::new(&response);

        assert_eq!(response.code, 200);
        assert_eq!(response.num_headers, 2);

        let owned = RackResponseOwned::from(response);
        assert_eq!(
            owned.headers.get("the year is 2024"),
            Some(&String::from("linux desktop is coming"))
        );
        assert_eq!(
            String::from_utf8_lossy(&owned.body),
            "apples and oranges".to_string()
        );
    }

    #[test]
    fn test_load_rails() {
        Ruby::load_app(&Path::new("tests/todo/config/environment.rb")).unwrap();
        let response = Ruby::eval("Rails.application.call({})").unwrap();
        let response = RackResponse::new(&response);
        let owned = RackResponseOwned::from(response);
        assert_eq!(owned.code, 403);
    }
}
