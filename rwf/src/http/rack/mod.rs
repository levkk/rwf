use libc::uintptr_t;
use std::ffi::{c_int, c_void, CString};
use std::os::raw::c_char;

use crate::error;

#[link(name = "ruby")]
extern "C" {
    fn ruby_setup() -> c_int;
    fn ruby_cleanup(code: c_int) -> c_int;
    fn rb_eval_string_protect(code: *const c_char, state: *mut c_int) -> uintptr_t;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Ruby VM did not start")]
    VmInit,

    #[error("Ruby VM state: {state}")]
    Eval { state: i64 },
}

pub struct Value {
    ptr: uintptr_t,
}

pub struct Ruby {}

impl Ruby {
    pub fn init() -> Result<(), Error> {
        unsafe {
            if ruby_setup() != 0 {
                Err(Error::VmInit)
            } else {
                Ok(())
            }
        }
    }

    pub fn eval(code: &str) -> Result<Value, Error> {
        unsafe {
            let mut state: c_int = 0;
            let c_string = CString::new(code).unwrap();
            let value = rb_eval_string_protect(c_string.as_ptr(), &mut state);

            if state != 0 {
                Err(Error::Eval {
                    state: state as i64,
                })
            } else {
                Ok(Value { ptr: value })
            }
        }
    }
}
