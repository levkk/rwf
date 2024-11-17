//! Conversion between URL parameters and Rust types.
use super::Error;
use std::fmt::Debug;

/// Convert a URL parameter to a Rust type.
pub trait ToParameter: Sync + Send + Debug {
    /// Perform the type conversion.
    fn to_parameter(s: &str) -> Result<Self, Error>
    where
        Self: Sized;
}

impl ToParameter for i64 {
    fn to_parameter(s: &str) -> Result<i64, Error> {
        match s.parse() {
            Ok(id) => Ok(id),
            Err(_) => Err(Error::MalformedRequest("i64")),
        }
    }
}

impl ToParameter for String {
    fn to_parameter(s: &str) -> Result<String, Error> {
        Ok(s.to_string())
    }
}
