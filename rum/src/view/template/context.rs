use crate::view::template::Value;
use std::collections::HashMap;

#[derive(Debug, Default)]
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
}
