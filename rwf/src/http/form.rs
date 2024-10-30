use super::{Error, FormData};

pub struct Form {
    data: Box<dyn FromFormData>,
}

pub trait FromFormData {
    fn from_form_data(form_data: &FormData) -> Result<Self, Error>
    where
        Self: Sized;
}

impl FromFormData for FormData {
    fn from_form_data(form_data: &FormData) -> Result<Self, Error> {
        Ok(form_data.clone())
    }
}
