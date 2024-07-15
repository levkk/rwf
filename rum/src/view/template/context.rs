use crate::view::template::{Error, ToValue, Value};
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Context {
    values: HashMap<String, Value>,
}

impl Context {
    pub fn new(values: HashMap<String, Value>) -> Self {
        Self { values }
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        self.values.get(key).cloned()
    }

    pub fn set(&mut self, key: &str, value: impl ToValue) -> Result<&mut Self, Error> {
        self.values.insert(key.to_string(), value.to_value()?);
        Ok(self)
    }
}
