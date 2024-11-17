//! Form parsing.
use super::{Error, FormData};

/// HTTP form.
pub struct Form {
    data: Box<dyn FromFormData>,
}

/// Handle conversion between HTML form data and a Rust struct.
pub trait FromFormData {
    /// Convert form data to Rust struct.
    fn from_form_data(form_data: &FormData) -> Result<Self, Error>
    where
        Self: Sized;
}

impl FromFormData for FormData {
    fn from_form_data(form_data: &FormData) -> Result<Self, Error> {
        Ok(form_data.clone())
    }
}
