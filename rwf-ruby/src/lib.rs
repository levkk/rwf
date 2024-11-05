use libc::uintptr_t;
use once_cell::sync::OnceCell;
use std::ffi::{c_int, CStr, CString};
use std::os::raw::c_char;

// Make sure the Ruby VM is initialized only once.
static RUBY_INIT: OnceCell<Ruby> = OnceCell::new();

#[link(name = "ruby")]
extern "C" {
    fn ruby_setup() -> c_int;
    fn ruby_cleanup(code: c_int) -> c_int;
    fn rb_errinfo() -> uintptr_t;

    // Execute some Ruby code.
    fn rb_eval_string_protect(code: *const c_char, state: *mut c_int) -> uintptr_t;
    fn rb_obj_as_string(value: uintptr_t) -> uintptr_t;
}

#[link(name = "rwf_ruby")]
extern "C" {
    fn rwf_rb_type(value: uintptr_t) -> c_int;

    fn rwf_value_cstr(value: uintptr_t) -> *mut c_char;

    fn rwf_clear_error_state();
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Ruby VM did not start")]
    VmInit,

    #[error("{err}")]
    Eval { err: String },
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
            if ruby_setup() != 0 {
                Err(Error::VmInit)
            } else {
                Ok(Ruby {})
            }
        }
    }

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
}

impl Drop for Ruby {
    fn drop(&mut self) {
        unsafe {
            ruby_cleanup(0);
        }
    }
}
